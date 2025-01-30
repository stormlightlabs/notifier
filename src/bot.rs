use crate::helpers;
use serde_json::Value;
use serenity::{all::Ready, model::id::ChannelId, prelude::*};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

struct Handler {
    channel_id: ChannelId,
    event_receiver: Arc<Mutex<Option<mpsc::Receiver<Value>>>>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let ctx = ctx.clone();
        let channel_id = self.channel_id;
        let rx_lock = self.event_receiver.clone();

        tokio::spawn(async move {
            let mut rx = rx_lock.lock().await.take().expect("Receiver already taken");

            while let Some(event) = rx.recv().await {
                let msg = format!(
                    "```json\n{}\n```",
                    serde_json::to_string_pretty(&event).unwrap()
                );
                if let Err(e) = channel_id.say(&ctx.http, msg).await {
                    eprintln!("Error sending message: {:?}", e);
                }
            }
        });
    }
}

pub async fn run_discord_bot(event_receiver: mpsc::Receiver<Value>) {
    let secrets = helpers::load_secrets().await;

    let channel_id = ChannelId::new(
        secrets
            .discord_channel_id
            .parse()
            .expect("Invalid channel ID"),
    );

    let bot = Handler {
        channel_id,
        event_receiver: Arc::new(Mutex::new(Some(event_receiver))),
    };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&secrets.discord_bot_token, intents)
        .event_handler(bot)
        .await
        .expect("error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
