#![allow(clippy::wildcard_imports)]
#![deny(clippy::unwrap_used)]

mod checks;
mod commands;
mod effects;
mod handlers;
mod hooks;
mod structs;
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
use tokio::signal::unix::{signal, SignalKind};
use tracing::{event, Level};

#[group]
#[commands(
    join,
    leave,
    pause,
    play,
    play_next,
    ping,
    resume,
    remove_at,
    shuffle,
    skip,
    stop,
    swap,
    now_playing,
    queue
)]
struct General;

#[tokio::main]
// allow unwrap_unused in main function (so during startup)
#[allow(clippy::unwrap_used)]
async fn main() {
    event!(Level::INFO, "Starting sunny");

    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Environment variable DISCORD_TOKEN not found");
    let app_id = env::var("APP_ID")
        .expect("Environment variable APP_ID not found")
        .parse()
        .expect("APP_ID needs to be a number");

    let cmd_prefix = env::var("CMD_PREFIX").expect("Environment variable CMD_PREFIX not found");

    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    let mut client = init_bot(token, app_id, cmd_prefix).await;
    let shard_manager = client.shard_manager.clone();

    select! {
        res = client.start() => match res {
            Err(err) => event!(Level::ERROR, %err, "client encountered an unexpected error"),
            _ => unreachable!()
        },
        res = tokio::signal::ctrl_c() => match res {
            Ok(()) => {
                event!(Level::INFO, "Received Ctrl-C, shutting down.");
                shard_manager.lock().await.shutdown_all().await;
            },
            Err(e) => event!(Level::ERROR, %e, "unable to listen for shutdown signal")
        },
        _ = sigterm.recv() => {
            event!(Level::INFO, "Received SIGTERM, shutting down.");
            shard_manager.lock().await.shutdown_all().await;
        },
    }
}

pub async fn init_bot(token: String, app_id: u64, cmd_prefix: String) -> Client {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(&cmd_prefix))
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
