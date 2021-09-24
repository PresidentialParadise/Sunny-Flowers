use std::{
    collections::HashSet,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use serenity::{
    client::Context,
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, misc::Mentionable, prelude::UserId},
    prelude::Mutex,
};
use songbird::input::restartable::Restartable;
use songbird::{Event, TrackEvent};

use crate::{
    handlers::{TimeoutHandler, TrackEndNotifier},
    utils::check_msg,
};

#[help]
pub async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, help_options, groups, owners)
        .await
        .unwrap();
    Ok(())
}

#[command]
#[only_in(guilds)]
/// Adds Sunny to the user's current voice channel.
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = if let Some(channel) = channel_id {
        channel
    } else {
        check_msg(msg.reply(ctx, "Not in a voice").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice Client placed in at initialisation")
        .clone();

    let (handle_lock, success) = manager.join(guild_id, connect_to).await;

    if let Ok(_channel) = success {
        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Joined {}", connect_to.mention()))
                .await,
        );

        let channel_id = msg.channel_id;
        let send_http = ctx.http.clone();
        let mut handle = handle_lock.lock().await;

        handle.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                channel_id,
                http: send_http,
            },
        );

        handle.add_global_event(
            Event::Periodic(Duration::from_secs(10), None),
            TimeoutHandler {
                guild_id,
                text_channel_id: channel_id,
                voice_channel_id: connect_to,
                timer: AtomicUsize::default(),
                ctx: ctx.clone(),
            },
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Failed to join channel")
                .await,
        );
    }

    deafen(handle_lock).await;

    Ok(())
}

async fn deafen(handler_lock: Arc<Mutex<songbird::Call>>) {
    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        println!("Client already deafened");
    } else if let Err(e) = handler.deafen(true).await {
        eprintln!("Failed: {:?}", e);
    }
}

#[command]
#[only_in(guilds)]
/// Removes Sunny from the current voice channel and clears the queue.
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice Client placed in at initialisation")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[aliases(p)]
#[max_args(1)]
#[only_in(guilds)]
#[usage("<url>")]
#[example("https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
/// While Sunny is in a voice channel, you may run the play command so that she
/// can start streaming the given video URL.
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = if let Ok(url) = args.single::<String>() {
        url
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Must provide a URL to a video or audio")
                .await,
        );

        return Ok(());
    };

    if !url.starts_with("http") {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Must provide a valid URL")
                .await,
        );

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice Client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source {:?}", why);
                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

        handler.enqueue_source(source.into());
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Added song to queue: position {}", handler.queue().len()),
                )
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
/// Skips the currently playing song and moves to the next song in the queue.
pub async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice Client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Song skipped: {} in queue.", queue.len()),
                )
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
/// Stops playing the current song and clears the current song queue.
pub async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice Client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        queue.stop();

        check_msg(msg.channel_id.say(&ctx.http, "Queue cleared.").await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
/// Pong
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong!").await);

    Ok(())
}
