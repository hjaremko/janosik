mod commands;
mod database;
mod runners;

use crate::commands::blackbox::BLACKBOX_GROUP;
use crate::commands::help::MY_HELP;
use crate::commands::protip::PROTIP_GROUP;
use crate::commands::send_message;
use crate::database::{Database, DatabaseConnection};
use serenity::futures::io::ErrorKind;
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
use tracing::{debug, error, info, instrument, warn, Level};
use tracing_subscriber::FmtSubscriber;

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
        info!("{} is connected!", ready.user.name);
    }
}

#[hook]
#[instrument]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );
    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => info!("Processed command '{}'", command_name),
        Err(why) => error!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    warn!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    debug!("{}: {}", msg.author.name, msg.content);

    const ABSURD: &str = r#"{
W związku z zapytaniem, informuję, iż problem z nagrywaniem wiąże się z
naruszeniem RODO wobec STUDENTÓW.

Dla mnie (na tę chwilę) jest to o tyle niezrozumiałe (mimo konkretnych
"ściśle prawniczych" argumentów podniesionych przez Panią Tokarczyk), że
nawet gdyby wszyscy studenci PROSILI o nagrywanie zajęć, to zgadzając się
na to, naruszamy RODO.

Absurd!!!
p.niemiec
    }"#;

    if msg.content != ABSURD && msg.content.to_lowercase().contains("nagrywa") {
        info!("Sending RODO notice");
        if let Err(e) = msg.channel_id.say(&_ctx.http, ABSURD).await {
            error!("Error sending RODO notice: {:?}", e);
        }
    }
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(duration) = error {
        let _ = send_message(
            ctx,
            msg,
            &format!("Try this again in {} seconds.", duration.as_secs()),
        );
    }
}

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref DB_MUTEX: Mutex<Database> = Mutex::new(Database::new());
}

type BoxError = Box<dyn std::error::Error>;
type BoxResult = Result<(), BoxError>;

#[tokio::main]
#[instrument]
async fn main() -> BoxResult {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO) //todo: set level info from command line arguments
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    {
        let mut db = DB_MUTEX.lock().await;
        db.connect()?;
        db.set_up()?;
    }

    if let Err(why) = make_client().await?.start().await {
        error!("Client error: {:?}", why);
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
        .group(&PROTIP_GROUP)
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
