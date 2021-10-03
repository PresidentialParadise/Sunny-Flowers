use std::cmp;

use serenity::{client::Context, model::id::GuildId};
use songbird::tracks::TrackHandle;

use crate::utils::{SunnyError, SunnyResult};

pub async fn swap(
    ctx: &Context,
    guild_id: GuildId,
    a: usize,
    b: usize,
) -> SunnyResult<(TrackHandle, TrackHandle)> {
    // What's this, a precondition, in my code!?
    if a == 0 || b == 0 {
        return Err(SunnyError::user("no"));
    }

    let call_m = songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?;

    let call = call_m.lock().await;

    let q = call.queue();

    if cmp::max(a, b) >= q.len() {
        return Err(SunnyError::user("no"));
    }

    let (t1, t2) = q.modify_queue(|q| {
        q.swap(a, b);
        (q[a].clone(), q[b].clone())
    });

    Ok((t1, t2))
}
