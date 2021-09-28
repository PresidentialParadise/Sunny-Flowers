#![allow(clippy::wildcard_imports)]

mod checks;
mod commands;
mod handlers;
mod hooks;
mod utils;

use std::env;
use std::process::exit;

use commands::*;
use hooks::{after_hook, dispatch_error_hook};

use dotenv::dotenv;

use handlers::Handler;
use serenity::{
    client::Client,
    framework::{standard::macros::group, StandardFramework},
};

use songbird::SerenityInit;

#[group]
#[commands(join, leave, play, ping, skip, stop, now_playing, queue)]
struct General;

#[tokio::main]
async fn main() {
    println!("Starting sunny");
    eprintln!("e: Starting sunny");

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
    let app_id = env::var("APP_ID")
        .expect("Environment variable APP_ID not found")
        .parse()
        .expect("APP_ID needs to be a number");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("?"))
        .group(&GENERAL_GROUP)
        .help(&HELP)
        .on_dispatch_error(dispatch_error_hook)
        .after(after_hook);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .application_id(app_id)
        .await
        .expect("Error creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| eprintln!("Client ended: {:?}", why));
        exit(1);
    });
}
