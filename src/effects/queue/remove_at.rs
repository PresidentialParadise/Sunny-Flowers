use std::num::NonZeroUsize;

use serenity::{client::Context, model::id::GuildId};
use songbird::tracks::Queued;
use tracing::instrument;

use crate::utils::{SunnyError, SunnyResult};

#[instrument(skip(ctx))]
pub async fn remove_at(ctx: &Context, guild_id: GuildId, at: NonZeroUsize) -> SunnyResult<Queued> {
    songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?
        .lock()
        .await
        .queue()
        .dequeue(at.into())
        .ok_or_else(|| SunnyError::user("Nothing to remove..."))
}
