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
        Args, CommandGroup, CommandResult, HelpOptions, Reason,
    },
    model::prelude::*,
    prelude::*,
};

use songbird::{input::restartable::Restartable, tracks::TrackHandle};
use songbird::{Event, TrackEvent};

use crate::{
    checks::*,
    handlers::{TimeoutHandler, TrackPlayNotifier},
    utils::{check_msg, generate_embed, generate_queue_embed},
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

    let connect_to = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or_else(|| Box::new(Reason::User("Not in a voice".to_string())))?;

    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?;

    let (call_m, success) = songbird.join(guild_id, connect_to).await;

    if success.is_ok() {
        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Joined {}", connect_to.mention()))
                .await,
        );

        let mut call = call_m.lock().await;

        call.remove_all_global_events();

        call.add_global_event(
            Event::Track(TrackEvent::Play),
            TrackPlayNotifier {
                channel_id: msg.channel_id,
                guild_id,
                ctx: ctx.clone(),
            },
        );

        call.add_global_event(
            Event::Periodic(Duration::from_secs(60), None),
            TimeoutHandler {
                guild_id,
                text_channel_id: msg.channel_id,
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

    deafen(call_m).await;

    Ok(())
}

async fn deafen(call_m: Arc<Mutex<songbird::Call>>) {
    let mut call = call_m.lock().await;

    if call.is_deaf() {
        println!("Client already deafened");
    } else if let Err(e) = call.deafen(true).await {
        eprintln!("Failed: {:?}", e);
    }
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Removes Sunny from the current voice channel and clears the queue.
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get guild".to_string())))?;
    let guild_id = guild.id;

    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?;

    songbird.remove(guild_id).await.map_err(|e| {
        Box::new(Reason::UserAndLog {
            user: "Failed to leave".to_string(),
            log: e.to_string(),
        })
    })?;

    check_msg(msg.channel_id.say(&ctx.http, "Left voice").await);

    Ok(())
}

#[command]
#[aliases(p)]
#[max_args(1)]
#[only_in(guilds)]
#[usage("<url>")]
#[example("https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
#[checks(In_Voice)]
/// While Sunny is in a voice channel, you may run the play command so that she
/// can start streaming the given video URL.
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url: String = args.single().map_err(|_| {
        Box::new(Reason::User(
            "Must provide a URL to a video or audio".to_string(),
        ))
    })?;

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

    let source = Restartable::ytdl(url, true).await.map_err(|e| {
        Box::new(Reason::UserAndLog {
            user: "Error starting stream".to_string(),
            log: format!("Error sourcing ffmpeg {:?}", e),
        })
    })?;

    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?;

    let call_m = songbird
        .get(guild_id)
        .ok_or_else(|| Box::new(Reason::Log("No Call".to_string())))?;

    let mut call = call_m.lock().await;

    call.enqueue_source(source.into());
    check_msg(
        msg.channel_id
            .say(
                &ctx.http,
                format!("Added song to queue: position {}", call.queue().len()),
            )
            .await,
    );

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(np)]
/// Shows the currently playing media
pub async fn now_playing(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let (current, track_list) = {
        let songbird = songbird::get(ctx)
            .await
            .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?;

        let call_m = songbird
            .get(guild.id)
            .ok_or_else(|| Box::new(Reason::Log("No Call".to_string())))?;

        let call = call_m.lock().await;

        (call.queue().current(), call.queue().current_queue())
    };

    let handle = current.ok_or_else(|| Box::new(Reason::User("No song playing".to_string())))?;

    let next_track = track_list.get(1).map(TrackHandle::metadata);

    let embed = generate_embed(handle.metadata(), next_track);
    check_msg(
        msg.channel_id
            .send_message(&ctx.http, |m| m.set_embed(embed))
            .await,
    );

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Skips the currently playing song and moves to the next song in the queue.
pub async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let call_m = songbird::get(ctx)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?
        .get(guild_id)
        .ok_or_else(|| Box::new(Reason::Log("No Call".to_string())))?;

    let call = call_m.lock().await;
    let queue = call.queue();
    let _ = queue.skip();

    check_msg(
        msg.channel_id
            .say(
                &ctx.http,
                format!(
                    "Song skipped: {} in queue.",
                    queue.len().checked_sub(1).unwrap_or_default()
                ),
            )
            .await,
    );
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Stops playing the current song and clears the current song queue.
pub async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    songbird::get(ctx)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?
        .get(guild_id)
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird call".to_string())))?
        .lock()
        .await
        .queue()
        .stop();

    check_msg(msg.channel_id.say(&ctx.http, "Queue cleared.").await);

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(q, queueueueu)]
/// Shows the current queue
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let cq = songbird::get(ctx)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?
        .get(guild_id)
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird call".to_string())))?
        .lock()
        .await
        .queue()
        .current_queue();

    let embed = generate_queue_embed(&cq, 0);
    check_msg(
        msg.channel_id
            .send_message(&ctx.http, |m| m.set_embed(embed))
            .await,
    );

    Ok(())
}

#[command]
#[only_in(guilds)]
/// Pong
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong!").await);
    Ok(())
}
