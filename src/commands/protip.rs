use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    utils::content_safe,
};

use crate::commands::locale::{
    add_protip_message, all_tasks_message, delete_protip_message, invalid_protip_id_message,
};
use crate::commands::{make_settings, send_message};
use crate::database::protip_handler::ProtipHandler;
use crate::{database, DATABASE};
use tracing::error;

#[group]
#[prefixes("protip")]
#[commands(add, remove, list)]
struct Protip;

#[command]
#[required_permissions(ADMINISTRATOR)]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (content, task) = parse_add_command(ctx, msg, args).await;

    send_message(&ctx, msg, &add_protip_message(&content, &task)).await?;

    if let Err(e) = DATABASE.add_protip(&task, &content).await {
        error!("Error adding protip: {:?}", e);
    }

    Ok(())
}

#[command]
#[required_permissions(ADMINISTRATOR)]
pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let protip_id = match args.single::<u32>() {
        Ok(id) => id,
        Err(_) => {
            send_message(&ctx, msg, &invalid_protip_id_message()).await?;
            return Ok(());
        }
    };

    if let Err(e) = DATABASE.remove_protip(protip_id).await {
        error!("Error removing protip: {:?}", e);
    }

    send_message(&ctx, msg, &delete_protip_message(&protip_id)).await?;
    Ok(())
}

#[command]
#[delimiters(' ')]
pub async fn list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let task = match args.current() {
        None => {
            let all_protips = DATABASE.get_tasks().await;
            send_message(&ctx, msg, &all_tasks_message(&all_protips)).await?;
            return Ok(());
        }
        Some(t) => t,
    };

    let protips = DATABASE.get_protip(task).await;
    send_message(&ctx, msg, &list_protips(&task, protips)).await?;

    Ok(())
}

pub fn list_protips(task: &str, protips: Vec<database::protip_handler::Protip>) -> String {
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
