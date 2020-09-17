use chrono::Utc;
use serenity::prelude::*;
use serenity::{
    async_trait,
    framework::standard::{macros::hook, CommandResult, DispatchError, StandardFramework},
    http::Http,
    model::{channel::Message, gateway::Ready, id::UserId},
    Error,
};
use std::collections::hash_map::RandomState;
use std::{collections::HashSet, env, io};

mod commands;

use crate::commands::blackbox::BLACKBOX_GROUP;
use crate::commands::help::MY_HELP;
use serenity::futures::io::ErrorKind;

#[macro_use]
extern crate scan_fmt;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        use serenity::model::gateway::Activity;
        use serenity::model::user::OnlineStatus;

        let activity = Activity::playing(format!("v{}", VERSION).as_str());
        let status = OnlineStatus::Online;

        context.set_presence(Some(activity), status).await;
        println!("{} is connected!", ready.user.name);
    }
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

type BoxError = Box<dyn std::error::Error>;
type BoxResult = Result<(), BoxError>;

#[tokio::main]
async fn main() -> BoxResult {
    if let Err(why) = make_client().await?.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

async fn make_client() -> Result<Client, Error> {
    let token = get_token_from_env()?;

    Client::new(&token)
        .event_handler(Handler)
        .framework(make_framework(get_owners(&token).await?))
        .await
}

fn make_framework(owners: HashSet<UserId>) -> StandardFramework {
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
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        .group(&BLACKBOX_GROUP)
}

async fn get_owners(token: &str) -> Result<HashSet<UserId, RandomState>, Error> {
    let http = Http::new_with_token(&token);

    match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();

            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }

            Ok(owners)
        }
        Err(why) => Err(why),
    }
}

fn get_token_from_env() -> Result<String, Error> {
    const TOKEN_NAME: &str = "JANOSIK_TOKEN";
    match env::var(TOKEN_NAME) {
        Ok(token) => Ok(token),
        Err(_) => {
            let string = format!("Variable {} is not present in the environment!", TOKEN_NAME);
            Err(Error::Io(io::Error::new(ErrorKind::NotFound, string)))
        }
    }
}
