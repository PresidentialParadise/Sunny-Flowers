use serenity::{client::Context, model::id::GuildId};

use crate::utils::{SunnyError, SunnyResult};

pub async fn skip(ctx: &Context, guild_id: GuildId) -> SunnyResult<usize> {
    let call_m = songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?;

    let call = call_m.lock().await;
    let queue = call.queue();

    queue.skip().map_err(|e| {
        SunnyError::user_and_log(
            "Failed to skip :shrug:",
            format!("Failed to skip: {}", e).as_str(),
        )
    })?;

    Ok(queue.len())
}
