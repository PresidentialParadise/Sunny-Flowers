use std::sync::atomic::{AtomicUsize, Ordering};

use serenity::{async_trait, model::prelude::*, prelude::*};

use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};

use crate::effects::{self, now_playing};
use crate::structs::EventConfig;
use crate::utils::Emitable;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let activity = Activity::streaming(
            "\u{1f4fb} Tropico News Today \u{1f9e8}",
            "https://www.youtube.com/watch?v=BmKMrUMS9lg",
        );

        let status = OnlineStatus::DoNotDisturb;

        ctx.set_presence(Some(activity), status).await;
    }
}

pub struct TrackPlayNotifier {
    pub cfg: EventConfig,
}

#[async_trait]
impl VoiceEventHandler for TrackPlayNotifier {
    async fn act(&self, event: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_track) = event {
            now_playing::send_embed(&self.cfg.ctx, self.cfg.guild_id, self.cfg.text_channel_id)
                .await
                .emit();
        }

        None
    }
}

pub struct TimeoutHandler {
    pub cfg: EventConfig,
    pub timer: AtomicUsize,
}

#[async_trait]
impl VoiceEventHandler for TimeoutHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let guild = if let Some(i) = self.cfg.ctx.cache.guild(self.cfg.guild_id).await {
            i
        } else {
            eprintln!("message guild id could not be found");
            return None;
        };
        if check_alone(
            &guild,
            self.cfg.voice_channel_id,
            self.cfg.ctx.cache.current_user_id().await,
        ) {
            let prev = self.timer.fetch_add(1, Ordering::Relaxed);

            if prev >= 5 {
                effects::leave(&self.cfg.ctx, self.cfg.guild_id)
                    .await
                    .emit();

                self.cfg
                    .text_channel_id
                    .say(&self.cfg.ctx.http, "Left voice due to lack of frens :(((")
                    .await
                    .emit();
            }
        } else {
            self.timer.store(0, Ordering::Relaxed);
        }

        None
    }
}

fn check_alone(guild: &Guild, channel_id: ChannelId, bot_id: UserId) -> bool {
    let mut states = guild.voice_states.values();

    !states.any(|vs| match vs.channel_id {
        Some(c_id) => channel_id.0 == c_id.0 && vs.user_id.0 != bot_id.0,
        None => false,
    })
}
