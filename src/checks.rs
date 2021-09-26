use serenity::{
    client::Context,
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::channel::Message,
    prelude::Mentionable,
};

#[check]
#[name = "In_Voice"]
#[display_in_help]
#[check_in_help]
// Ensures a command is only usable if in the same voice channel as sunny
pub async fn in_same_voice_check(
    ctx: &Context,
    msg: &Message,
    _args: &mut Args,
    _command_options: &CommandOptions,
) -> Result<(), Reason> {
    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| Reason::Log("Failed to get songbird".to_string()))?;

    let guild_id = msg
        .guild_id
        .ok_or_else(|| Reason::Log("Guild ID Empty".to_string()))?;

    let channel = {
        let songbird_call_m = songbird
            .get(guild_id)
            .ok_or_else(|| Reason::User("Not currently in a call".to_string()))?;

        let songbird_call = songbird_call_m.lock().await;

        songbird_call
            .current_channel()
            .ok_or_else(|| Reason::Log("Couldn't find songbird channel".to_string()))?
    };

    let name = serenity::model::id::ChannelId(channel.0);

    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| Reason::Log("Couldn't get guild".to_string()))?;

    let mut states = guild.voice_states.values();

    states
        .any(|vs| match vs.channel_id {
            Some(c_id) => channel.0 == c_id.0 && vs.user_id.0 == msg.author.id.0,
            None => false,
        })
        .then(|| ())
        .ok_or_else(|| {
            Reason::User(format!(
                "I only take requests from users in {}",
                name.mention()
            ))
        })
}
