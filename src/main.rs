mod commands;
mod handlers;
mod utils;

use std::env;

#[allow(clippy::wildcard_imports)]
use commands::*;

use dotenv::dotenv;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::prelude::Ready,
};

use songbird::SerenityInit;

#[group]
#[commands(join, leave, play, ping, skip, stop)]
struct General;

#[tokio::main]
async fn main() {
    create_bot().await;
    match tokio::signal::ctrl_c().await {
        Ok(()) => println!("Received Ctrl-C, shutting down."),
        Err(e) => {
            eprintln!("Unable to listen for shutdown signal {}", e);
        }
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
    }
}

pub async fn create_bot() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Environment variable DISCORD_TOKEN not found");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP)
        .help(&HELP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| println!("Client ended: {:?}", why));
    });
}
