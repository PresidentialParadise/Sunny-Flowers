#![allow(clippy::wildcard_imports)]

mod commands;
mod handlers;
mod utils;

use std::env;

use commands::*;

use dotenv::dotenv;

use handlers::Handler;
use serenity::{
    client::Client,
    framework::{standard::macros::group, StandardFramework},
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
