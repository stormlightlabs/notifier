//! Discord Bot implementation
use std::sync::Arc;

use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct _DiscordMessage {
    channel_id: u64,
    content: String,
}

#[derive(Clone)]
pub struct State {
    pub _sender: broadcast::Sender<Message>,
}

pub struct Handler {
    _state: Arc<State>,
    pub receiver: broadcast::Receiver<Message>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let mut rx = self.receiver.resubscribe();
        let ctx = Arc::new(ctx);

        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Err(why) = ChannelId::new(msg.channel_id.get())
                    .say(&ctx.http, msg.content)
                    .await
                {
                    println!("Error sending message: {:?}", why);
                }
            }
        });
    }
}
