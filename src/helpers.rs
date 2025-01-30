use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tokio::time;

#[derive(Debug, Clone)]
pub struct ImproperConfigError;

impl std::fmt::Display for ImproperConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "channel id not found")
    }
}

pub async fn ticker(opt_duration: Option<time::Duration>) {
    let mut interval = time::interval(time::Duration::from_secs(10));

    if let Some(duration) = opt_duration {
        interval = time::interval(duration);
    }

    loop {
        tracing::info!("heartbeat");
        interval.tick().await;
    }
}

/// Loads environment variables from a .env file
///
/// # Arguments
/// * `path` - Path to the .env file
///
/// # Returns
/// * `Result<HashMap<String, String>, std::io::Error>` - Map of environment variables or error
///
/// # Example
/// ```rust
/// let env_vars = load_env_file(".env").unwrap();
/// ```
pub fn load_env_file<P: AsRef<Path>>(path: P) -> std::io::Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut env_vars = HashMap::new();

    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = parse_env_line(&line) {
            env_vars.insert(key.to_string(), value.to_string());
            env::set_var(key, value);
        }
    }

    Ok(env_vars)
}

/// Parses a single environment variable line
///
/// # Arguments
/// * `line` - Line from .env file
///
/// # Returns
/// * `Option<(&str, &str)>` - Tuple of (key, value) if valid
fn parse_env_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '=');

    let key = parts.next()?.trim();
    let value = parts.next()?.trim();

    if key.is_empty() || value.is_empty() {
        return None;
    }

    let value = value.trim_matches(|c| c == '"' || c == '\'');

    Some((key, value))
}

#[derive(Clone)]
pub struct Secrets {
    pub discord_bot_token: String,
    pub discord_channel_id: String,
    pub webhook_secret: String,
}

pub async fn load_secrets() -> Secrets {
    let _ = load_env_file(".env");

    let discord_bot_token =
        env::var("DISCORD_BOT_TOKEN").expect("expected a token for the discord bot");

    let discord_channel_id = env::var("DISCORD_CHANNEL_ID")
        .expect("expected to retrieve a channel id for the configured server");

    let webhook_secret =
        env::var("GITHUB_WEBHOOK_SECRET").expect("env var GITHUB_WEBHOOK_SECRET must be set");

    Secrets {
        discord_channel_id,
        discord_bot_token,
        webhook_secret,
    }
}
