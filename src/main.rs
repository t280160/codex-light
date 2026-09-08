use codex_light::codex::{human_duration, resolve_codex_home, UsageService};

#[tokio::main]
async fn main() {
    let json_output = std::env::args().skip(1).any(|arg| arg == "--json");

    let result = match resolve_codex_home() {
        Ok(home) => UsageService::new(home).get_usage().await,
        Err(error) => Err(error),
    };

    match result {
        Ok(usage) if json_output => match serde_json::to_string_pretty(&usage) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("Unable to encode usage: {error}");
                std::process::exit(1);
            }
        },
        Ok(usage) => {
            println!("Codex detected\n");
            print_window("5h", usage.five_hour_remaining, usage.five_hour_reset_at);
            print_window("Weekly", usage.weekly_remaining, usage.weekly_reset_at);
            println!("Source: {}", usage.source);
            println!("Last updated: {}", usage.updated_at);
            if usage.stale {
                println!("Status: last known usage (stale)");
            }
        }
        Err(error) if json_output => {
            let body = serde_json::json!({ "error": error.user_message() });
            println!(
                "{}",
                serde_json::to_string_pretty(&body).unwrap_or_default()
            );
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("{}", error.user_message());
            std::process::exit(1);
        }
    }
}

fn print_window(label: &str, remaining: u8, reset_at: Option<i64>) {
    println!("{label}: {remaining}% remaining");
    if let Some(reset_at) = reset_at {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_default();
        println!("Reset in {}", human_duration(reset_at.saturating_sub(now)));
    } else {
        println!("Reset time unavailable");
    }
    println!();
}
