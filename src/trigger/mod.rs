mod rodo;

use crate::commands::send_message;
use crate::trigger::rodo::Rodo;
use rand::Rng;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::{debug, error, info};

pub trait Trigger {
    fn message() -> String;
    fn name() -> String;
    fn frequency() -> i32;
    fn contains_trigger(content: &str) -> bool;
}

pub async fn _handle_triggers(ctx: &Context, msg: &Message) {
    _handle_trigger::<Rodo>(ctx, &msg).await;
}

async fn _handle_trigger<T: Trigger>(ctx: &Context, msg: &Message) {
    if _is_not_sent_by_bot::<T>(msg) && T::contains_trigger(&msg.content) && _is_lucky::<T>() {
        info!("Sending {}", T::name());

        if let Err(e) = send_message(ctx, msg, &T::message()).await {
            error!("Error sending {}: {:?}", T::name(), e);
        }
    }
}

fn _is_lucky<T: Trigger>() -> bool {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..100);

    debug!("{} dice roll: {}", T::name(), roll);

    roll <= T::frequency() % 100
}

fn _is_not_sent_by_bot<T: Trigger>(msg: &Message) -> bool {
    msg.content != T::message()
}
