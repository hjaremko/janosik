use crate::commands::locale::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    utils::{content_safe, ContentSafeOptions},
};

use crate::runners::binary_runner::BinaryRunner;
use crate::runners::runner_error::RunnerError;
use serenity::utils::MessageBuilder;
use tracing::{debug, info};

#[group]
#[commands(blackbox)]
struct Blackbox;

#[command]
pub async fn blackbox(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (content, program_name, input) = parse_blackbox_command(ctx, msg, args).await;

    debug!("Content: {}", content);
    info!("Program: {}", program_name);
    info!("Input: {}", input);

    let output = match BinaryRunner::run(&program_name, &input) {
        Ok(out) => out,
        Err(e) => match e {
            RunnerError::NoInput => no_input_message(),
            RunnerError::Timeout => timeout_message(&program_name),
            RunnerError::NotFound => not_found_message(&program_name),
            RunnerError::NoOutput => no_output_message(&program_name),
            RunnerError::Crash => crash_message(&program_name),
            RunnerError::Other(e) => e,
        },
    };

    let content = MessageBuilder::new().user(&msg.author).build();
    send_message(&ctx, msg, &format!("{}\n{}", content, output)).await?;
    Ok(())
}

async fn send_message(ctx: &Context, msg: &Message, content: &str) -> serenity::Result<Message> {
    info!("Sent: {}", content);
    msg.channel_id.say(&ctx.http, content).await
}

async fn parse_blackbox_command(
    ctx: &Context,
    msg: &Message,
    args: Args,
) -> (String, String, String) {
    let content = trim_content(ctx, msg, &args).await;
    let (program_name, input) = extract_program_name_and_input(&content);

    (content, program_name, input)
}

fn extract_program_name_and_input(content: &str) -> (String, String) {
    let (program_name, input) = scan_fmt_some!(&content, "{} {/```(.*)```/}", String, String);
    (program_name.unwrap(), remove_ticks_or_empty(input))
}

fn remove_ticks_or_empty(str: Option<String>) -> String {
    str.map_or(String::new(), |c| c.replace('`', ""))
}

async fn trim_content(ctx: &Context, msg: &Message, args: &Args) -> String {
    content_safe(&ctx.cache, &args.rest(), &make_settings(msg))
        .await
        .replace('\n', " ")
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
