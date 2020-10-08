use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::utils::ContentSafeOptions;
use tracing::debug;

pub mod blackbox;
pub mod help;
pub mod locale;
pub mod protip;

pub async fn send_message(
    ctx: &Context,
    msg: &Message,
    content: &str,
) -> serenity::Result<Message> {
    debug!("Sent: {}", content);
    msg.channel_id.say(&ctx.http, content).await
}

pub fn make_settings(msg: &Message) -> ContentSafeOptions {
    if let Some(guild_id) = msg.guild_id {
        ContentSafeOptions::default()
            .clean_channel(false)
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    }
}
