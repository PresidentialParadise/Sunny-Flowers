use std::{
    collections::HashSet,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use serenity::{
    builder::CreateActionRow,
    client::Context,
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args, CommandGroup, CommandResult, HelpOptions, Reason,
    },
    futures::prelude::*,
    model::{interactions::message_component::ButtonStyle, prelude::*},
    prelude::*,
};

use songbird::{input::restartable::Restartable, Event, TrackEvent};

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

pub async fn send_now_playing_embed(
    ctx: &Context,
    channel_id: ChannelId,
    guild_id: GuildId,
) -> Result<(), Box<Reason>> {
    let (current, next) = {
        let songbird = songbird::get(ctx)
            .await
            .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?;

        let call_m = songbird
            .get(guild_id)
            .ok_or_else(|| Box::new(Reason::Log("No Call".to_string())))?;

        let call = call_m.lock().await;

        (
            call.queue().current(),
            call.queue().current_queue().get(1).cloned(),
        )
    };

    let current = current.ok_or_else(|| Box::new(Reason::User("No song playing".to_string())))?;

    let position = current
        .get_info()
        .await
        .map_err(|e| Box::new(Reason::Log(format!("a: {:?}", e))))? // * hackers on diefstal e(stradiol)n he
        .position;

    let next_metadata = next.map(|t| t.metadata().clone());

    // e
    let mut m = channel_id
        .send_message(&ctx.http, |m| {
            m.set_embed(generate_embed(
                current.metadata(),
                position,
                next_metadata.as_ref(),
            ))
        })
        .await
        .map_err(|estradiol| {
            Box::new(Reason::Log(format!(
                "Sending message failed {:?}",
                estradiol
            )))
        })?;

    let c = ctx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            // Will error when finished
            if let Ok(info) = current.get_info().await {
                let embed =
                    generate_embed(current.metadata(), info.position, next_metadata.as_ref());

                m.edit(&c.http, |e| e.set_embed(embed)).await.ok();
            } else {
                break;
            }
        }
    });

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(np)]
/// Shows the currently playing media
pub async fn now_playing(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    send_now_playing_embed(ctx, msg.channel_id, msg.guild_id.unwrap()).await?;
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
    const PREV_ID: &str = "q_prev";
    const NEXT_ID: &str = "q_next";

    fn build_action_row(page: usize, queue_len: usize) -> CreateActionRow {
        let pages = queue_len / 10;
        let mut row = CreateActionRow::default();

        // Previous button
        if page > 0 {
            row.create_button(|b| {
                b.style(ButtonStyle::Primary);
                b.label("Previous");
                b.custom_id(PREV_ID);
                b.disabled(false)
            });
        } else {
            row.create_button(|b| {
                b.style(ButtonStyle::Danger);
                b.label("Previous");
                b.custom_id(PREV_ID);
                b.disabled(true)
            });
        }

        // Next button
        if pages >= 1 && page < pages {
            row.create_button(|b| {
                b.style(ButtonStyle::Primary);
                b.label("Next");
                b.custom_id(NEXT_ID);
                b.disabled(false)
            });
        } else {
            row.create_button(|b| {
                b.style(ButtonStyle::Danger);
                b.label("Next");
                b.custom_id(NEXT_ID);
                b.disabled(true)
            });
        }

        row
    }

    let guild_id = msg.guild_id.unwrap();

    // Retrieve the current queue
    let cq = songbird::get(ctx)
        .await
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird".to_string())))?
        .get(guild_id)
        .ok_or_else(|| Box::new(Reason::Log("Couldn't get songbird call".to_string())))?
        .lock()
        .await
        .queue()
        .current_queue();

    // Currently shown page
    let mut page = 0;

    // Send initial queue message
    let mut message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.components(|c| c.set_action_rows(vec![build_action_row(page, cq.len())]));
            m.set_embed(generate_queue_embed(&cq, page))
        })
        .await
        .map_err(|e| {
            Box::new(Reason::Log(format!(
                "Unable to send queue message: {:?}",
                e
            )))
        })?;

    // await interactions i.e. button presses
    let mut collector = message
        .await_component_interactions(&ctx.shard)
        .timeout(Duration::from_secs(3 * 60))
        .await;

    // Process button presses
    while let Some(mci) = collector.next().await {
        if mci.data.custom_id == NEXT_ID {
            page += 1;
        } else if mci.data.custom_id == PREV_ID {
            page -= 1;
        } else {
            continue;
        }

        // Change the embed + buttons after page change
        mci.create_interaction_response(&ctx.http, |cir| {
            cir.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|m| {
                    m.add_embed(generate_queue_embed(&cq, page));
                    m.components(|c| c.set_action_rows(vec![build_action_row(page, cq.len())]))
                })
        })
        .await
        .map_err(|e| {
            Box::new(Reason::Log(format!(
                "Unable to create interaction response: {:?}",
                e
            )))
        })?;
    }

    // Remove buttons after timeout
    message
        .edit(&ctx.http, |e| {
            e.components(|c| c);
            e.set_embed(generate_queue_embed(&cq, page))
        })
        .await
        .map_err(|e| Box::new(Reason::Log(format!("Unable clear buttons {:?}", e))))?;

    Ok(())
}

#[command]
#[only_in(guilds)]
/// Pong
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong!").await);
    Ok(())
}
