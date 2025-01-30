use serenity::all::{ChannelId, GuildId};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImproperConfigError;

impl std::fmt::Display for ImproperConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "channel id not found")
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

pub async fn get_channel_id() -> Result<(ChannelId, String), ImproperConfigError> {
    let env_map = load_env_file(".env").unwrap();

    let server_id = u64::from_str_radix(
        env_map
            .get("DISCORD_SERVER_ID")
            .expect("Github Server ID is required"),
        10,
    )
    .expect("discord_server_id should be a defined and valid 64 bit integer");

    let discord_token = env_map
        .get("DISCORD_BOT_TOKEN")
        .expect("Expected a valid token for the discord bot in the environment")
        .clone();
    let client = serenity::http::Http::new(&discord_token);

    match client.get_guild(GuildId::new(server_id)).await {
        Ok(guild) => {
            let channels = guild
                .channels(client)
                .await
                .expect("unable to fetch channel list for guild");

            for (channel_id, channel) in channels {
                if channel.name.contains("github") {
                    return Ok((channel_id, discord_token));
                }
            }
            eprintln!("channel with github not found");

            Err(ImproperConfigError)
        }
        Err(val) => {
            eprintln!("unable to fetch guild: {val}");

            Err(ImproperConfigError)
        }
    }
}

pub async fn load_secrets() -> Secrets {
    let (channel_id, discord_bot_token) = get_channel_id()
        .await
        .expect("expected to retrieve a channel id for the configured server");

    let _ = env::var("CLOUDFLARE_TUNNEL_TOKEN")
        .expect("expected a token for the cloudflare ingress in the environment");

    let _ = env::var("DISCORD_APPLICATION_KEY").unwrap();

    let webhook_secret =
        env::var("GITHUB_WEBHOOK_SECRET").expect("env var GITHUB_WEBHOOK_SECRET must be set");

    Secrets {
        discord_channel_id: channel_id.to_string(),
        discord_bot_token,
        webhook_secret,
    }
}
