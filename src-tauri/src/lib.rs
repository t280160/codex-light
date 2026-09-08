#[path = "../../src/codex/mod.rs"]
mod codex;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{Mutex, Notify};

use codex::{resolve_codex_home, CodexUsage, UsageError, UsageService};
use serde::{Deserialize, Serialize};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WebviewWindow, WindowEvent,
};

const DEFAULT_REFRESH_INTERVAL_MINUTES: u64 = 5;
const REFRESH_INTERVAL_OPTIONS: [u64; 6] = [1, 5, 10, 15, 30, 60];

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    refresh_interval_minutes: u64,
}

struct AppState {
    codex_home: PathBuf,
    refresh_lock: Arc<Mutex<()>>,
    refresh_interval_minutes: AtomicU64,
    refresh_schedule_changed: Arc<Notify>,
    settings_path: PathBuf,
}

pub fn run() {
    let settings_path = settings_path();
    let refresh_interval_minutes = load_refresh_interval(&settings_path);

    tauri::Builder::default()
        .manage(AppState {
            codex_home: resolve_codex_home().unwrap_or_else(|_| {
                dirs::home_dir().map(|home| home.join(".codex")).unwrap_or_else(|| PathBuf::from(".codex"))
            }),
            refresh_lock: Arc::new(Mutex::new(())),
            refresh_interval_minutes: AtomicU64::new(refresh_interval_minutes),
            refresh_schedule_changed: Arc::new(Notify::new()),
            settings_path,
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.handle().set_activation_policy(tauri::ActivationPolicy::Accessory)?;

            let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit = PredefinedMenuItem::quit(app, Some("Quit"))?;
            let menu = Menu::with_items(app, &[&refresh, &separator, &quit])?;

            TrayIconBuilder::with_id("codex-light")
                .icon(tray_image(None))
                .title("--")
                .tooltip("CodexLight")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        show_popup(tray.app_handle());
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "refresh" => {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = refresh_from_app(&handle).await;
                        });
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let show_on_launch = std::env::var_os("CODEXLIGHT_SHOW_WINDOW").is_some();
                configure_popup(&window, show_on_launch);
                if show_on_launch {
                    show_popup(app.handle());
                }
            }
            start_background_refresh(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_codex_usage,
            refresh_codex_usage,
            get_refresh_interval_minutes,
            set_refresh_interval_minutes
        ])
        .run(tauri::generate_context!())
        .expect("error while running CodexLight");
}

fn configure_popup(window: &WebviewWindow, show_on_launch: bool) {
    if !show_on_launch {
        let _ = window.hide();
    }
    let popup = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = popup.hide();
        }
    });
}

fn show_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = refresh_from_app(&handle).await;
    });
}

fn start_background_refresh(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(error) = refresh_from_app(&app).await {
                eprintln!("CodexLight background refresh failed: {error}");
            }

            let (minutes, schedule_changed) = {
                let state = app.state::<AppState>();
                (
                    state.refresh_interval_minutes.load(Ordering::Relaxed),
                    state.refresh_schedule_changed.clone(),
                )
            };

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(minutes * 60)) => {}
                _ = schedule_changed.notified() => {}
            }
        }
    });
}

#[tauri::command]
fn get_refresh_interval_minutes(state: State<'_, AppState>) -> u64 {
    state.refresh_interval_minutes.load(Ordering::Relaxed)
}

#[tauri::command]
fn set_refresh_interval_minutes(minutes: u64, state: State<'_, AppState>) -> Result<u64, String> {
    let minutes = validate_refresh_interval(minutes)
        .ok_or_else(|| "Refresh interval must be 1, 5, 10, 15, 30, or 60 minutes".to_string())?;

    save_refresh_interval(&state.settings_path, minutes)
        .map_err(|error| format!("Unable to save refresh interval: {error}"))?;
    state.refresh_interval_minutes.store(minutes, Ordering::Relaxed);
    state.refresh_schedule_changed.notify_waiters();
    Ok(minutes)
}

#[tauri::command]
async fn get_codex_usage(app: AppHandle) -> Result<CodexUsage, String> {
    refresh_from_app(&app).await
}

#[tauri::command]
async fn refresh_codex_usage(app: AppHandle) -> Result<CodexUsage, String> {
    refresh_from_app(&app).await
}

async fn refresh_from_app(app: &AppHandle) -> Result<CodexUsage, String> {
    let state = app.state::<AppState>();
    let _guard = state
        .refresh_lock
        .lock()
        .await;
    let usage = UsageService::new(state.codex_home.clone())
        .get_usage()
        .await
        .map_err(|error: UsageError| error.user_message())?;
    update_tray(app, &usage);
    let _ = app.emit("codex-usage-updated", usage.clone());
    Ok(usage)
}

fn update_tray(app: &AppHandle, usage: &CodexUsage) {
    if let Some(tray) = app.tray_by_id("codex-light") {
        let _ = tray.set_icon(Some(tray_image(Some(usage.five_hour_remaining))));
        let _ = tray.set_title(Some(format!("{}%", usage.five_hour_remaining)));
        let _ = tray.set_tooltip(Some(format!("CodexLight · {}% remaining", usage.five_hour_remaining)));
    }
}

fn tray_image(remaining: Option<u8>) -> Image<'static> {
    let (red, green, blue) = match remaining {
        Some(value) if value >= 50 => (36, 184, 112),
        Some(value) if value >= 20 => (224, 157, 27),
        Some(_) => (228, 76, 85),
        None => (139, 153, 167),
    };
    let size = 32usize;
    let center = 15.5f32;
    let radius = 11.5f32;
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let distance = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            if distance <= radius {
                let index = (y * size + x) * 4;
                rgba[index..index + 4].copy_from_slice(&[red, green, blue, 255]);
            }
        }
    }
    Image::new_owned(rgba, size as u32, size as u32)
}

fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("CodexLight")
        .join("settings.json")
}

fn load_refresh_interval(path: &Path) -> u64 {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str::<AppSettings>(&contents).ok())
        .and_then(|settings| validate_refresh_interval(settings.refresh_interval_minutes))
        .unwrap_or(DEFAULT_REFRESH_INTERVAL_MINUTES)
}

fn save_refresh_interval(path: &Path, minutes: u64) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(&AppSettings {
        refresh_interval_minutes: minutes,
    })
    .map_err(std::io::Error::other)?;
    fs::write(path, contents)
}

fn validate_refresh_interval(minutes: u64) -> Option<u64> {
    REFRESH_INTERVAL_OPTIONS.contains(&minutes).then_some(minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_interval_accepts_only_supported_options() {
        for minutes in REFRESH_INTERVAL_OPTIONS {
            assert_eq!(validate_refresh_interval(minutes), Some(minutes));
        }
        assert_eq!(validate_refresh_interval(0), None);
        assert_eq!(validate_refresh_interval(2), None);
        assert_eq!(validate_refresh_interval(120), None);
    }
}
