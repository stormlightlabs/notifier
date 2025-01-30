use std::sync::Arc;

use server::{create_listener, create_service};
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod bot;
mod helpers;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (event_sender, rx) = mpsc::channel(100);
    let secrets = helpers::load_secrets().await;

    tracing::info!("created communication channel & loaded secrets");

    tokio::spawn(bot::run_discord_bot(rx));
    tokio::spawn(helpers::ticker(None));

    let state = Arc::new(server::SharedState {
        event_sender,
        secrets,
    });

    let service = create_service(state);
    let listener = create_listener(4040).await;

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, service).await.unwrap();
}
