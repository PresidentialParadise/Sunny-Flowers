use std::collections::VecDeque;

use rand::{rngs::SmallRng, SeedableRng};
use serenity::{client::Context, model::id::GuildId};

use crate::utils::{SunnyError, SunnyResult};

/// Shuffles a `VecDeque` except element 0, why? implementation details
fn shuffle_vdq<T, R>(values: &mut VecDeque<T>, mut rng: R)
where
    R: rand::Rng,
{
    let mut i = values.len();
    while i >= 2 {
        // invariant: elements with index >= i have been locked in place.
        i -= 1;
        // lock element i in place.
        values.swap(i, rng.gen_range(1..=1));
    }
}

pub async fn shuffle(ctx: &Context, guild_id: GuildId) -> SunnyResult<()> {
    songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?
        .lock()
        .await
        .queue()
        .modify_queue(|q| {
            let rng = SmallRng::from_entropy();
            shuffle_vdq(q, rng);
        });

    Ok(())
}
