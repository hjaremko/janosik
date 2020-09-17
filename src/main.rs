use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{command, group, help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, id::UserId},
    utils::{content_safe, ContentSafeOptions},
};
use std::{collections::HashSet, env, fs};

use chrono::Utc;
use serenity::prelude::*;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[group]
#[commands(blackbox)]
struct Blackbox;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[help]
#[individual_command_tip = "BaCa assignments blackbox"]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );
    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!(
        "[{}] {}: {}",
        Utc::now().format("%T"),
        msg.author.name,
        msg.content
    );
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(duration) = error {
        let _ = msg
            .channel_id
            .say(
                &ctx.http,
                &format!("Try this again in {} seconds.", duration.as_secs()),
            )
            .await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("JANOSIK_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let mut client = Client::new(token)
        .event_handler(Handler)
        .framework(
            StandardFramework::new()
                .configure(|c| {
                    c.with_whitespace(true)
                        .prefix("!")
                        .delimiters(vec![", ", ","])
                        .owners(owners)
                })
                .before(before)
                .after(after)
                .unrecognised_command(unknown_command)
                .normal_message(normal_message)
                // .on_dispatch_error(dispatch_error)
                .help(&MY_HELP)
                .group(&BLACKBOX_GROUP),
        )
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[macro_use]
extern crate scan_fmt;

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

#[command]
async fn blackbox(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
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
