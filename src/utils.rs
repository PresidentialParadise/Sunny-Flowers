use serenity::{model::channel::Message, Result as SerenityResult};
use songbird::input::Metadata;

pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

pub fn generate_embed(m: &Metadata) -> serenity::builder::CreateEmbed {
    let mut e = serenity::builder::CreateEmbed::default();

    e.author(|a| a.name("Now Playing:"));

    let title = if let Some(track) = &m.track {
        track
    } else if let Some(title) = &m.title {
        title
    } else {
        "Unknown Title"
    };

    let artist = if let Some(artist) = &m.artist {
        artist
    } else if let Some(channel) = &m.channel {
        channel
    } else {
        "Unknown Artist"
    };

    e.title(format!("{} by {}", title, artist));

    if let Some(thumbnail) = &m.thumbnail {
        e.thumbnail(thumbnail);
    }

    if let Some(url) = &m.source_url {
        e.url(url);
    }

    e.timestamp(&chrono::Utc::now());

    e
}
