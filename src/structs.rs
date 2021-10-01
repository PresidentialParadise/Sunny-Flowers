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
