use std::sync::Arc;

use server::{create_listener, create_service};
use tokio::sync::mpsc;

mod bot;
mod helpers;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (event_sender, rx) = mpsc::channel(100);
    let secrets = helpers::load_secrets().await;
    tracing::info!("created communication channel & loaded secrets");

    tokio::spawn(bot::run_discord_bot(rx));

    let state = Arc::new(server::SharedState {
        event_sender,
        secrets,
    });

    let service = create_service(state);
    let listener = create_listener(4040).await;

    axum::serve(listener, service).await.unwrap();
}
