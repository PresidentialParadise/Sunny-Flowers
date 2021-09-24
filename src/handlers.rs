use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serenity::{
    async_trait,
    client::Context,
    http::Http,
    model::{
        guild::Guild,
        id::{ChannelId, GuildId, UserId},
    },
};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};

use crate::utils::check_msg;

pub struct TrackEndNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            check_msg(
                self.channel_id
                    .say(&self.http, format!("Tracks ended: {}", track_list.len()))
                    .await,
            );
        }

        None
    }
}

pub struct TimeoutHandler {
    pub guild_id: GuildId,
    pub voice_channel_id: ChannelId,
    pub text_channel_id: ChannelId,
    pub timer: AtomicUsize,
    pub ctx: Context,
}

#[async_trait]
impl VoiceEventHandler for TimeoutHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let guild = self.ctx.cache.guild(self.guild_id).await.unwrap();
        if check_alone(
            &guild,
            self.voice_channel_id,
            self.ctx.cache.current_user_id().await,
        ) {
            let prev = self.timer.fetch_add(1, Ordering::Relaxed);

            if prev >= 5 {
                let manager = songbird::get(&self.ctx)
                    .await
                    .expect("Songbird Voice Client placed in at initialisation")
                    .clone();

                if let Err(e) = manager.remove(self.guild_id).await {
                    eprintln!("Failed: {:?}", e);
                }

                check_msg(
                    self.text_channel_id
                        .say(&self.ctx.http, "Left voice due to lack of frens :(((")
                        .await,
                );
            }
        } else {
            let _ = self.timer.swap(0, Ordering::Relaxed);
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
