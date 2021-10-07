use serenity::{client::Context, model::id::GuildId};
use tracing::instrument;

use crate::utils::{SunnyError, SunnyResult};

#[instrument(skip(ctx))]
pub async fn pause(ctx: &Context, guild_id: GuildId) -> SunnyResult<()> {
    songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?
        .lock()
        .await
        .queue()
        .current()
        .ok_or_else(|| SunnyError::user("No track playing"))?
        .pause()
        .map_err(|e| {
            SunnyError::user_and_log(
                "Failed to pause :person_shrugging:",
                format!("Failed to pause: {}", e).as_str(),
            )
        })
}
