use serenity::model::prelude::Message;
use serenity::prelude::Context;
use tracing::info;

pub mod blackbox;
pub mod help;
pub mod locale;
pub mod protip;

pub async fn send_message(
    ctx: &Context,
    msg: &Message,
    content: &str,
) -> serenity::Result<Message> {
    info!("Sent: {}", content);
    msg.channel_id.say(&ctx.http, content).await
}
