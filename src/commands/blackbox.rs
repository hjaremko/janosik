use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    utils::{content_safe, ContentSafeOptions},
};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read};
use std::process::{Command, Stdio};
use std::time::Duration;
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

    fs::write("tmp.txt", input).expect("Unable to write file");
    let file = File::open("tmp.txt")?;

    let child = Command::new(&format!("bin/{}", program_name))
        .stdin(Stdio::from(file))
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(_) => child.unwrap(),
        Err(_) => {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("Nie znaleziono ` {} `", program_name))
                .await
            {
                println!("Error sending message: {:?}", why);
            }

            panic!("Invalid program name");
        }
    };

    println!("Program {} started", program_name);

    let timeout = Duration::from_secs(30);
    let status_code = match child.wait_timeout(timeout).unwrap() {
        Some(status) => status.code(),
        None => {
            child.kill().unwrap();

            if let Err(why) = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!(
                        "`{}` działał zbyt długo, sprawdź poprawność wejścia",
                        program_name
                    ),
                )
                .await
            {
                println!("Error sending message: {:?}", why);
            }

            panic!("Time out");
        }
    };

    println!("{} returned {:?}", program_name, status_code);

    if status_code.is_none() || status_code.unwrap() != 0 {
        if let Err(why) = msg
            .channel_id
            .say(
                &ctx.http,
                format!("`{}` wyjebał się, sprawdź poprawność wejścia", program_name),
            )
            .await
        {
            println!("Error sending message: {:?}", why);
        }

        panic!("Crash");
    }

    let stdout = child
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let mut reader = BufReader::new(stdout);
    let mut out = String::new();

    if let Err(why) = reader.read_to_string(&mut out) {
        println!("Error reading output: {:?}", why);
    };

    out = if out.is_empty() {
        format!(
            "`{}` nic nie wypisał, sprawdź poprawność wejścia",
            program_name
        )
    } else {
        format!("```\n{}\n```", out)
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, &out).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

async fn parse_blackbox_command(
    ctx: &Context,
    msg: &Message,
    args: Args,
) -> (String, String, String) {
    let settings = if let Some(guild_id) = msg.guild_id {
        ContentSafeOptions::default()
            .clean_channel(false)
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings)
        .await
        .replace('\n', " ");
    let (program_name, code) = scan_fmt_some!(&content, "{} {/```(.*)```/}", String, String);
    let input = code.map_or(String::new(), |c| c.replace('`', ""));
    let program_name = program_name.unwrap();

    (content, program_name, input)
}
