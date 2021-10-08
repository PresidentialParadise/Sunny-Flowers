use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use once_cell::sync::Lazy;
use serenity::prelude::Mutex;
use songbird::{Call, Event, TrackEvent};
use tracing::instrument;

use crate::{
    handlers::{TimeoutHandler, TrackPlayNotifier},
    structs::EventConfig,
    utils::{SunnyError, SunnyResult},
};

static IS_CONNECTING: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[instrument]
async fn add_events(cfg: &EventConfig, call_m: Arc<Mutex<Call>>) {
    let mut call = call_m.lock().await;
    call.remove_all_global_events();

    call.add_global_event(
        Event::Track(TrackEvent::Play),
        TrackPlayNotifier { cfg: cfg.clone() },
    );

    call.add_global_event(
        Event::Periodic(Duration::from_secs(60), None),
        TimeoutHandler {
            timer: AtomicUsize::default(),
            cfg: cfg.clone(),
        },
    );
}

#[instrument]
pub async fn join(cfg: &EventConfig) -> SunnyResult<Arc<Mutex<Call>>> {
    let songbird = songbird::get(&cfg.ctx)
        .await
        .ok_or_else(|| SunnyError::log("Couldn't get songbird"))?;

    let guard = IS_CONNECTING.lock().await;

    let (call_m, success) = songbird.join(cfg.guild_id, cfg.voice_channel_id).await;

    drop(guard);

    success
        .map_err(|e| SunnyError::user_and_log("Failed to join channel", e.to_string().as_str()))?;

    add_events(cfg, call_m.clone()).await;

    Ok(call_m)
}
