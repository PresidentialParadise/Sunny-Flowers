use serenity::{
    model::{channel::Message, guild::Guild, id::UserId},
    Result as SerenityResult,
};
use songbird::{id::ChannelId, input::Metadata};

pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

pub fn check_user_in_voice(guild: &Guild, channel_id: ChannelId, user_id: UserId) -> bool {
    let mut states = guild.voice_states.values();

    states.any(|vs| match vs.channel_id {
        Some(c_id) => channel_id.0 == c_id.0 && vs.user_id.0 == user_id.0,
        None => false,
    })
}

pub fn generate_embed(m: &Metadata, m2: Option<&Metadata>) -> serenity::builder::CreateEmbed {
    let mut e = serenity::builder::CreateEmbed::default();

    e.author(|a| a.name("Now Playing:"));

    let title = m
        .track
        .as_deref()
        .or_else(|| m.title.as_deref())
        .unwrap_or("Unknown Title");

    let artist = m
        .artist
        .as_deref()
        .or_else(|| m.channel.as_deref())
        .unwrap_or("Unknown Artist");

    e.title(format!("{} by {}", title, artist));

    if let Some(thumbnail) = &m.thumbnail {
        e.thumbnail(thumbnail);
    }

    if let Some(url) = &m.source_url {
        e.url(url);
    }

    if let Some(m2) = m2 {
        let title2 = m2
            .track
            .as_deref()
            .or_else(|| m2.title.as_deref())
            .unwrap_or("Unknown Title");

        let artist2 = m2
            .artist
            .as_deref()
            .or_else(|| m2.channel.as_deref())
            .unwrap_or("Unknown Artist");

        e.description(format!("**Up Next:** {} by {}", title2, artist2));
    }

    e.timestamp(&chrono::Utc::now());

    e
}
