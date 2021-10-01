use std::sync::atomic::{AtomicUsize, Ordering};

use serenity::{async_trait, model::prelude::*, prelude::*};

use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};

use crate::commands::send_now_playing_embed;
use crate::utils::check_msg;

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

    async fn voice_state_update(
        &self,
        ctx: Context,
        _: Option<GuildId>,
        _: Option<VoiceState>,
        voice_state: VoiceState,
    ) {
        let current_user_id = ctx.cache.current_user_id().await;

        // If the state update does not concern us: ignore
        if voice_state.user_id != current_user_id {
            return;
        }

        // If our new state doesn't have a voice channel i.e. if we have been forcefully disconnected
        if voice_state.channel_id.is_none() {
            let guild_id = voice_state.guild_id.unwrap();

            let songbird = songbird::get(&ctx).await.unwrap();

            if songbird.get(guild_id).is_some() {
                if let Err(err) = songbird.remove(guild_id).await {
                    eprintln!(
                        "Error removing Sunny from songbird after state update {:?}",
                        err
                    );
                }
            }

            println!("left succesfully after force disconnect");
        }
    }
}

pub struct TrackPlayNotifier {
    pub channel_id: ChannelId,
    pub ctx: Context,
    pub guild_id: GuildId,
}

#[async_trait]
impl VoiceEventHandler for TrackPlayNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_track) = ctx {
            if let Err(diefstal_en_heling) =
                send_now_playing_embed(&self.ctx, self.channel_id, self.guild_id).await
            {
                eprintln!("Kinderen herhaal: {}", diefstal_en_heling);
            }
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
                let songbird = songbird::get(&self.ctx)
                    .await
                    .expect("Songbird Voice Client placed in at initialisation")
                    .clone();

                if let Err(e) = songbird.remove(self.guild_id).await {
                    eprintln!("Failed: {:?}", e);
                }

                check_msg(
                    self.text_channel_id
                        .say(&self.ctx.http, "Left voice due to lack of frens :(((")
                        .await,
                );
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
