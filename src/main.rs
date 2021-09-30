#![allow(clippy::wildcard_imports)]

mod checks;
mod commands;
mod handlers;
mod hooks;
mod utils;

use std::env;

use commands::*;
use hooks::{after_hook, dispatch_error_hook};

use dotenv::dotenv;

use handlers::Handler;
use serenity::{
    client::Client,
    framework::{standard::macros::group, StandardFramework},
};

use tokio::select;

use songbird::SerenityInit;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;

#[group]
#[commands(join, leave, play, ping, skip, stop, now_playing, queue)]
struct General;

#[tokio::main]
async fn main() {
    println!("Starting sunny");
    eprintln!("e: Starting sunny");

    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Environment variable DISCORD_TOKEN not found");
    let app_id = env::var("APP_ID")
        .expect("Environment variable APP_ID not found")
        .parse()
        .expect("APP_ID needs to be a number");

    let mut stream = signal(SignalKind::terminate()).unwrap();

    let mut client = init_bot(token, app_id).await;
    let shard_manager = client.shard_manager.clone();

    select! {
        res = client.start() => match res {
            Err(e) => eprintln!("Client had a fuckywucky OwO Penultimo is wowking hawd to fricks it uwu O_o {}", e),
            _ => unreachable!()
        },
        res = tokio::signal::ctrl_c() => match res {
            Ok(()) => {
                println!("Received Ctrl-C, shutting down.");
                shard_manager.lock().await.shutdown_all().await;
            },
            Err(e) => eprintln!("Unable to listen for shutdown signal {}", e)
        },
        _ = stream.recv() => {
            println!("Received SIGTERM, shutting down.");
            shard_manager.lock().await.shutdown_all().await;
        },
    }
}

pub async fn init_bot(token: String, app_id: u64) -> Client {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("?"))
        .group(&GENERAL_GROUP)
        .help(&HELP)
        .on_dispatch_error(dispatch_error_hook)
        .after(after_hook);

    Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .application_id(app_id)
        .await
        .expect("Error creating client")
}
