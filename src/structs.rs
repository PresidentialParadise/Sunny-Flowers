use std::fmt::Debug;

use serenity::{
    client::Context,
    model::id::{ChannelId, GuildId},
};

#[derive(Clone)]
pub struct EventConfig {
    pub ctx: Context,
    pub guild_id: GuildId,
    pub text_channel_id: ChannelId,
    pub voice_channel_id: ChannelId,
}

impl Debug for EventConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventConfig")
            .field("guild_id", &self.guild_id)
            .field("text_channel_id", &self.text_channel_id)
            .field("voice_channel_id", &self.voice_channel_id)
            .finish()
    }
}
