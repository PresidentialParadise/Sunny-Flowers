use serenity::{client::Context, model::id::GuildId};
use songbird::input::Restartable;

use crate::utils::{SunnyError, SunnyResult};

pub enum EnqueueAt {
    Front,
    Back,
}

pub async fn play(
    ctx: &Context,
    guild_id: GuildId,
    url: String,
    enqueu_at: EnqueueAt,
) -> SunnyResult<usize> {
    let source = Restartable::ytdl(url, true).await.map_err(|e| {
        SunnyError::user_and_log(
            "Error starting stream",
            format!("Error sourcing ffmpeg {:?}", e).as_str(),
        )
    })?;

    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?;

    let call_m = songbird
        .get(guild_id)
        .ok_or_else(|| SunnyError::log("No Call"))?;

    let mut call = call_m.lock().await;

    match enqueu_at {
        EnqueueAt::Front => {
            call.enqueue_source(source.into());
            call.queue().modify_queue(|q| {
                if let Some(track) = q.pop_back() {
                    q.push_front(track);
                    if q.len() > 1 {
                        q.swap(0, 1);
                    }
                }
            });
        }
        EnqueueAt::Back => call.enqueue_source(source.into()),
    };
    Ok(call.queue().len())
}
