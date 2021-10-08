use serenity::{client::Context, model::id::GuildId};
use tracing::instrument;

use crate::utils::{SunnyError, SunnyResult};

#[instrument(skip(ctx))]
pub async fn leave(ctx: &Context, guild_id: GuildId) -> SunnyResult<()> {
    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get Songbird"))?;

    songbird
        .remove(guild_id)
        .await
        .map_err(|e| SunnyError::user_and_log("Failed to leave", e.to_string().as_str()))?;

    Ok(())
}
