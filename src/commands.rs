use std::collections::HashSet;

use serenity::{
    client::Context,
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::prelude::*,
};

use url::Url;

use crate::{
    checks::*,
    effects::{self, now_playing, queue},
    structs::EventConfig,
    utils::SunnyError,
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
        .ok_or_else(|| SunnyError::log("failed to send"))?;
    Ok(())
}

#[command]
#[only_in(guilds)]
/// Adds Sunny to the user's current voice channel.
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    let voice_channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or_else(|| SunnyError::user("Not in a voice"))?;

    let call_m = effects::join(&EventConfig {
        ctx: ctx.clone(),
        guild_id: guild.id,
        text_channel_id: msg.channel_id,
        voice_channel_id,
    })
    .await?;

    msg.channel_id
        .say(&ctx.http, format!("Joined {}", voice_channel_id.mention()))
        .await?;

    effects::deafen(call_m).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Removes Sunny from the current voice channel and clears the queue.
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get guild"))?;

    effects::leave(ctx, guild.id).await?;

    msg.channel_id.say(&ctx.http, "Left voice").await?;

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
    let mut url: String = args
        .single()
        .map_err(|_| SunnyError::user("Must provide a URL to a video or audio"))?;

    if url.starts_with('<') && url.ends_with('>') {
        url = url[1..url.len() - 1].to_string();
    }

    if Url::parse(&url).is_err() {
        return Err(SunnyError::user("Must provide a valid URL").into());
    }

    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    let len = effects::play(ctx, guild_id, url).await?;

    msg.channel_id
        .say(&ctx.http, format!("Added song to queue: position {}", len))
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(np)]
/// Shows the currently playing media
pub async fn now_playing(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    now_playing::send_embed(ctx, guild_id, msg.channel_id).await?;

    msg.delete(&ctx.http).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Skips the currently playing song and moves to the next song in the queue.
pub async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    let len = effects::skip(ctx, guild_id).await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "Song skipped: {} in queue.",
                len.checked_sub(1).unwrap_or_default()
            ),
        )
        .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(In_Voice)]
/// Stops playing the current song and clears the current song queue.
pub async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    effects::stop(ctx, guild_id).await?;

    msg.channel_id.say(&ctx.http, "Queue cleared.").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(q, queueueueu)]
/// Shows the current queue
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    queue::send_embed(ctx, guild_id, msg.channel_id).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
/// Pong
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;
    Ok(())
}
