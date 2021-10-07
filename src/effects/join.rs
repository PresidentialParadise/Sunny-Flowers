use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use serenity::prelude::Mutex;
use songbird::{Call, Event, TrackEvent};
use tracing::instrument;

use crate::{
    handlers::{TimeoutHandler, TrackPlayNotifier},
    structs::EventConfig,
    utils::{SunnyError, SunnyResult},
};

static IS_CONNECTING: AtomicBool = AtomicBool::new(false);

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

    IS_CONNECTING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .map_err(|_| SunnyError::log("some witty log message"))?;

    let (call_m, success) = songbird.join(cfg.guild_id, cfg.voice_channel_id).await;

    // ! Deadlock if panic
    IS_CONNECTING.store(false, Ordering::SeqCst);

    success
        .map_err(|e| SunnyError::user_and_log("Failed to join channel", e.to_string().as_str()))?;

    add_events(cfg, call_m.clone()).await;

    Ok(call_m)
}
