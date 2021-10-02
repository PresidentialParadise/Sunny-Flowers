use std::time::Duration;

use serenity::{
    builder::CreateActionRow,
    client::Context,
    futures::prelude::*,
    model::{
        channel::Message,
        id::{ChannelId, GuildId},
        interactions::{message_component::ButtonStyle, InteractionResponseType},
    },
};
use songbird::tracks::TrackHandle;

use crate::utils::{SunnyError, SunnyResult};

use super::*;

const PREV_ID: &str = "q_prev";
const NEXT_ID: &str = "q_next";

fn generate_embed(queue: &[TrackHandle], page: usize) -> serenity::builder::CreateEmbed {
    let mut titles = Vec::with_capacity(10);
    let mut artists = Vec::with_capacity(10);
    let mut durs = Vec::with_capacity(10);
    let total_duration = queue.iter().fold(Duration::default(), |a, b| {
        a + b.metadata().duration.unwrap_or_default()
    });

    for (i, track) in queue.iter().enumerate().skip(page * 10).take(10) {
        let m = track.metadata();

        let title = format!("**{}.** {}\n", i + 1, get_title(m));
        titles.push(title);

        let artist = format!("{}\n", get_artist(m));
        artists.push(artist);

        let duration = m.duration.unwrap_or_default();
        let seconds = duration.as_secs() % 60;
        let minutes = duration.as_secs() / 60;

        durs.push(format!("[{}:{:02}]\n", minutes, seconds));
    }

    let mut e = serenity::builder::CreateEmbed::default();
    e.author(|a| a.name("Queueueueueu"));

    if let Some(track) = queue.get(1) {
        let m = track.metadata();

        e.description(format!(
            "**Up Next:** {} by {}",
            get_title(m),
            get_artist(m)
        ));
    }

    // * Change to intersperse after #79524 stablizes
    e.field(
        "# Title",
        string_or_default(titles.into_iter().collect::<String>().trim_end(), "Queue"),
        true,
    );

    e.field(
        "Artist",
        string_or_default(artists.into_iter().collect::<String>().trim_end(), "is"),
        true,
    );

    e.field(
        "Duration",
        string_or_default(durs.into_iter().collect::<String>().trim_end(), "empty"),
        true,
    );

    e.footer(|f| {
        let seconds = total_duration.as_secs() % 60;
        let minutes = total_duration.as_secs() / 60;

        f.text(format!(
            "Page {}/{} | Total Duration: {:02}:{:02}",
            page + 1,
            (queue.len() / 10 + 1),
            minutes,
            seconds,
        ))
    });

    e
}

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

async fn get_queue(ctx: &Context, guild_id: GuildId) -> SunnyResult<Vec<TrackHandle>> {
    Ok(songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::user("Not currently in a call"))?
        .lock()
        .await
        .queue()
        .current_queue())
}

/// Sends an interactive queue embed and interactions
pub async fn send_embed(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> SunnyResult<()> {
    // Retrieve the current queue
    let cq = get_queue(ctx, guild_id).await?;

    // Send initial queue message
    let message = channel_id
        .send_message(&ctx.http, |m| {
            m.components(|c| c.set_action_rows(vec![build_action_row(0, cq.len())]));
            m.set_embed(generate_embed(&cq, 0))
        })
        .await
        .map_err(|e| SunnyError::log(format!("Unable to send queue message: {:?}", e).as_str()))?;

    await_interactions(ctx, message, guild_id).await
}

async fn await_interactions(
    ctx: &Context,
    mut msg: Message,
    guild_id: GuildId,
) -> SunnyResult<()> {
    // Currently shown page
    let mut page: usize = 0;

    // await interactions i.e. button presses
    let mut collector = msg
        .await_component_interactions(&ctx.shard)
        .timeout(Duration::from_secs(3600)) // 1h
        .await;

    // Process button presses
    while let Some(mci) = collector.next().await {
        if mci.data.custom_id == NEXT_ID {
            page += 1;
        } else if mci.data.custom_id == PREV_ID {
            page = if let Some(p) = page.checked_sub(1) {
                p
            } else {
                continue;
            };
        } else {
            continue;
        }

        let cq = get_queue(ctx, guild_id).await?;

        // Change the embed + buttons after page change
        mci.create_interaction_response(&ctx.http, |cir| {
            cir.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|m| {
                    m.add_embed(generate_embed(&cq, page));
                    m.components(|c| c.set_action_rows(vec![build_action_row(page, cq.len())]))
                })
        })
        .await
        .map_err(|e| {
            SunnyError::log(format!("Unable to create interaction response: {:?}", e).as_str())
        })?;
    }

    let guild_id = msg.guild_id
        .ok_or_else(|| SunnyError::log("message guild id could not be found"))?;

    let cq = get_queue(ctx, guild_id).await?;

    // Remove buttons after timeout
    msg
        .edit(&ctx.http, |e| {
            e.components(|c| c);
            e.set_embed(generate_embed(&cq, page))
        })
        .await
        .map_err(|e| SunnyError::log(format!("Unable clear buttons {:?}", e).as_str()))?;

    Ok(())
}
