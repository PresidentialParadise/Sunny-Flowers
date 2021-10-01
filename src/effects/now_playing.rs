use std::time::Duration;

use serenity::{
    client::Context,
    model::id::{ChannelId, GuildId},
};
use songbird::{input::Metadata, tracks::TrackHandle};

use crate::utils::{SunnyError, SunnyResult};

use super::{get_artist, get_title, split_duration};

/// Generates an embed to show what's currently playing and what is up next
pub fn generate_embed(
    m: &Metadata,
    pos: Duration,
    m2: Option<&Metadata>,
) -> serenity::builder::CreateEmbed {
    let mut e = serenity::builder::CreateEmbed::default();

    e.author(|a| a.name("Now Playing:"));

    let title = get_title(m);

    let artist = get_artist(m);

    e.title(format!("{} by {}", title, artist));

    if let Some(thumbnail) = &m.thumbnail {
        e.thumbnail(thumbnail);
    }

    if let Some(url) = &m.source_url {
        e.url(url);
    }

    let (curr_min, curr_sec) = split_duration(pos);
    let (max_min, max_sec) = split_duration(m.duration.unwrap_or_default());
    let progress = format!(
        "**Current Time:** {}:{:02} / {}:{:02}",
        curr_min, curr_sec, max_min, max_sec
    );

    let up_next = m2
        .map(|m2| format!("**Up Next:** {} by {}", get_title(m2), get_artist(m2)))
        .unwrap_or_default();

    e.description(&[progress, up_next].join("\n"));
    e.timestamp(&chrono::Utc::now());

    e
}

/// Gets the current and next up song's [`TrackHandle`].
async fn get_songs(
    ctx: &Context,
    guild_id: GuildId,
) -> SunnyResult<(Option<TrackHandle>, Option<TrackHandle>)> {
    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?;

    let call_m = songbird
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?;

    let call = call_m.lock().await;

    Ok((
        call.queue().current(),
        call.queue().current_queue().get(1).cloned(),
    ))
}

/// Sends a `now_playing` embed and updates the progress every 10 seconds
pub async fn send_embed(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> SunnyResult<()> {
    let (current, next) = get_songs(ctx, guild_id).await?;

    let current = current.ok_or_else(|| SunnyError::user("No song playing"))?;

    let position = current
        .get_info()
        .await
        .map_err(|e| SunnyError::log(format!("a: {:?}", e).as_str()))? // * hackers on diefstal e(stradiol)n heling
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
            SunnyError::log(format!("Sending message failed {:?}", estradiol).as_str())
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
                m.delete(&c.http).await.unwrap();
                break;
            }
        }
    });

    Ok(())
}
