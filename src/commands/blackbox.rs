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
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::Duration;
use std::{fs, io};
use wait_timeout::ChildExt;

#[group]
#[commands(blackbox)]
struct Blackbox;

#[command]
pub async fn blackbox(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (content, program_name, input) = parse_blackbox_command(ctx, msg, args).await;

    println!("Content: {}", content);
    println!("Program: {}", program_name);
    println!("Input: {}", input);

    let (child, status_code) = match run_program(&program_name, input).await {
        Ok(c) => c,
        Err(e) => {
            send_message(&ctx, msg, e.as_str()).await?;
            return Ok(());
        }
    };

    if status_code != 0 {
        send_message(&ctx, msg, crash_message(&program_name).as_str()).await?;
        return Ok(()); // todo: figure out how to return Err
    }

    let output = read_output(program_name, child);

    send_message(&ctx, msg, output.as_str()).await?;
    Ok(())
}

fn read_output(program_name: String, child: Child) -> String {
    let mut reader = BufReader::new(get_stdout(child).unwrap());
    let mut output = String::new();

    reader.read_to_string(&mut output).unwrap();

    output = if output.is_empty() {
        no_output_message(program_name.as_str())
    } else {
        format!("```\n{}\n```", output)
    };
    output
}

async fn run_program(program_name: &str, input: String) -> Result<(Child, i32), String> {
    let file = match create_tmp_file(input) {
        Ok(f) => f,
        Err(_) => return Err("Cannot open tmp file".to_string()),
    };
    let child = spawn_process(&program_name, file);

    if child.is_err() {
        return Err(not_found_message(&program_name));
    }

    let mut child = child.unwrap();

    println!("Program {} started", program_name);

    let timeout = Duration::from_secs(30);

    let status_code = match child.wait_timeout(timeout) {
        Ok(c) => c,
        Err(_) => return Err("IO error".to_string()),
    };

    let status_code = match status_code {
        None => {
            child.kill().expect("Command wasn't running");
            return Err(timeout_message(&program_name));
        }
        Some(status) => status.code().unwrap(),
    };

    println!("{} returned {:?}", program_name, status_code);
    Ok((child, status_code))
}

fn get_stdout(child: Child) -> Result<ChildStdout, Error> {
    child
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))
}

async fn send_message(ctx: &Context, msg: &Message, content: &str) -> serenity::Result<Message> {
    msg.channel_id.say(&ctx.http, content).await
}

fn spawn_process(program_name: &str, file: File) -> Result<Child, Error> {
    Command::new(format!("bin/{}", program_name))
        .stdin(Stdio::from(file))
        .stdout(Stdio::piped())
        .spawn()
}

fn create_tmp_file(input: String) -> io::Result<File> {
    const TMP_FILENAME: &str = "tmp.txt";
    fs::write(TMP_FILENAME, input).expect("cannot write to tmp file");
    File::open(TMP_FILENAME)
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
