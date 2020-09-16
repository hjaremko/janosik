use std::{collections::HashSet, fs, env};
use serenity::{
    framework::standard::{
        Args, CommandResult, CommandGroup,
        HelpOptions, help_commands, StandardFramework,
        macros::{command, group, help},
    },
    model::{channel::Message, gateway::Ready, id::UserId},
    utils::{content_safe, ContentSafeOptions},
};

use serenity::prelude::*;
use std::process::{Command, Stdio};
use std::io::{BufReader, Error, ErrorKind, Read};
use std::fs::File;
use std::time::Duration;
use wait_timeout::ChildExt;
use chrono::Utc;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

group!({
    name: "blackbox",
    options: {
        description: "BaCa Blackbox",
    },
    commands: [blackbox],
});

#[help]
#[individual_command_tip =
"BaCa assignments blackbox"]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

fn main() {
    let token = env::var("JANOSIK_TOKEN").expect(
        "Expected a token in the environment",
    );

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    let (owners, _bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c
                .with_whitespace(true)
                .prefix("!")
                .delimiters(vec![", ", ","])
                .owners(owners))

            .before(|_ctx, msg, command_name| {
                println!("Got command '{}' by user '{}'", command_name, msg.author.name);
                true
            })
            .after(|_, _, command_name, error| {
                match error {
                    Ok(()) => println!("Processed command '{}'", command_name),
                    Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
                }
            })
            .unrecognised_command(|_, _, unknown_command_name| {
                println!("Could not find command named '{}'", unknown_command_name);
            })
            .normal_message(|_, message| {
                println!("[{}] {}: {}", Utc::now().format("%T"), message.author.name, message.content);
            })
            .help(&MY_HELP)
            .group(&BLACKBOX_GROUP)
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

#[macro_use]
extern crate scan_fmt;

fn parse_blackbox_command(ctx: &mut Context, msg: &Message, args: Args) -> (String, String, String)
{
    let settings = if let Some(guild_id) = msg.guild_id {
        ContentSafeOptions::default()
            .clean_channel(false)
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings).replace('\n', " ");
    let (program_name, code) = scan_fmt_some!(&content, "{} {/```(.*)```/}", String, String);
    let input = code.map_or(String::new(), |c| c.replace('`', ""));
    let program_name = program_name.unwrap();

    (content, program_name, input)
}

#[command]
fn blackbox(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let (content, program_name, input) = parse_blackbox_command(ctx, msg, args);

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
        Ok(_) => { child.unwrap() }
        Err(_) => {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http,
                     format!("Nie znaleziono ` {} `", program_name)) {
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

            if let Err(why) = msg.channel_id.say(&ctx.http,
                                                 &format!("`{}` działał zbyt długo, sprawdź poprawność wejścia"
                                                          , program_name)) {
                println!("Error sending message: {:?}", why);
            }

            panic!("Time out");
        }
    };

    println!("{} returned {:?}", program_name, status_code);

    if status_code.is_none() || status_code.unwrap() != 0 {
        if let Err(why) = msg
            .channel_id
            .say(&ctx.http,
                 format!("`{}` wyjebał się, sprawdź poprawność wejścia",
                         program_name)) {
            println!("Error sending message: {:?}", why);
        }

        panic!("Crash");
    }

    let stdout = child.stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let mut reader = BufReader::new(stdout);
    let mut out = String::new();

    if let Err(why) = reader.read_to_string(&mut out) {
        println!("Error reading output: {:?}", why);
    };

    out = if out.is_empty() {
        format!("`{}` nic nie wypisał, sprawdź poprawność wejścia", program_name)
    } else {
        format!("```\n{}\n```", out)
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, &out) {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}
