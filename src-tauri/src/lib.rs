#[path = "../../src/codex/mod.rs"]
mod codex;

use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use codex::{resolve_codex_home, CodexUsage, UsageError, UsageService};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewWindow, WindowEvent,
};

const BACKGROUND_REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);

struct AppState {
    codex_home: PathBuf,
    refresh_lock: Arc<Mutex<()>>,
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            codex_home: resolve_codex_home().unwrap_or_else(|_| {
                dirs::home_dir().map(|home| home.join(".codex")).unwrap_or_else(|| PathBuf::from(".codex"))
            }),
            refresh_lock: Arc::new(Mutex::new(())),
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
        .invoke_handler(tauri::generate_handler![get_codex_usage, refresh_codex_usage])
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
            tokio::time::sleep(BACKGROUND_REFRESH_INTERVAL).await;
        }
    });
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
