use serenity::{client::Context, model::id::GuildId};

use crate::utils::{SunnyError, SunnyResult};

pub async fn stop(ctx: &Context, guild_id: GuildId) -> SunnyResult<()> {
    songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("Couldn't get songbird call"))?
        .lock()
        .await
        .queue()
        .stop();

    Ok(())
}
