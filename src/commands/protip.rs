use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    utils::{content_safe, ContentSafeOptions},
};

use crate::commands::locale::{
    add_protip_message, all_tasks_message, delete_protip_message, invalid_protip_id_message,
};
use crate::commands::send_message;
use crate::database::ProtipHandler;
use crate::{database, DB_MUTEX};

#[group]
#[prefixes("protip")]
#[commands(add, remove, list)]
struct Protip;

#[command]
#[required_permissions(ADMINISTRATOR)]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (content, task) = parse_add_command(ctx, msg, args).await;

    send_message(&ctx, msg, &add_protip_message(&content, &task)).await?;
    let mut db = DB_MUTEX.lock().await;
    db.add_protip(&task, &content)?;

    Ok(())
}

#[command]
#[required_permissions(ADMINISTRATOR)]
pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut db = DB_MUTEX.lock().await;

    let protip_id = match args.single::<u32>() {
        Ok(id) => id,
        Err(_) => {
            send_message(&ctx, msg, &invalid_protip_id_message()).await?;
            return Ok(());
        }
    };

    db.remove_protip(protip_id)?;
    send_message(&ctx, msg, &delete_protip_message(&protip_id)).await?;
    Ok(())
}

#[command]
#[delimiters(' ')]
pub async fn list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut db = DB_MUTEX.lock().await;

    let task = match args.current() {
        None => {
            let all_protips = db.get_tasks();
            send_message(&ctx, msg, &all_tasks_message(&all_protips)).await?;
            return Ok(());
        }
        Some(t) => t,
    };

    let protips = db.get_protip(task);
    send_message(&ctx, msg, &list_protips(&task, protips)).await?;

    Ok(())
}

pub fn list_protips(task: &str, protips: Vec<database::Protip>) -> String {
    let mut header = format!("Protipy `{}`:\n", task);

    for protip in protips {
        header.push_str(&format!("\t{}\n", &protip));
    }

    header
}

async fn parse_add_command(ctx: &Context, msg: &Message, args: Args) -> (String, String) {
    let content = trim_content(ctx, msg, &args).await;
    let (content, task) = extract_task_and_content(&content);

    (content, task)
}

async fn trim_content(ctx: &Context, msg: &Message, args: &Args) -> String {
    content_safe(&ctx.cache, &args.rest(), &make_settings(msg)).await
}

fn extract_task_and_content(content: &str) -> (String, String) {
    let (task, content) = scan_fmt_some!(&content, "{} {/(.*)/}", String, String);
    (content.unwrap(), task.unwrap())
}

fn make_settings(msg: &Message) -> ContentSafeOptions {
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
