use serenity::{model::channel::Message, Result as SerenityResult};
use songbird::{input::Metadata, tracks::TrackHandle};

pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

pub fn generate_embed(m: &Metadata, m2: Option<&Metadata>) -> serenity::builder::CreateEmbed {
    let mut e = serenity::builder::CreateEmbed::default();

    e.author(|a| a.name("Now Playing:"));

    let title = get_title(m);

    let artist = get_artist(m);

    e.title(format!("{} by {}", title, artist));

    if let Some(thumbnail) = &m.thumbnail {
        e.thumbnail(thumbnail);
    }

    if let Some(url) = &m.source_url {
        e.url(url);
    }

    if let Some(m2) = m2 {
        let title2 = get_title(m2);
        let artist2 = get_artist(m2);

        e.description(format!("**Up Next:** {} by {}", title2, artist2));
    }

    e.timestamp(&chrono::Utc::now());

    e
}

pub fn generate_queue_embed(
    queue: Vec<TrackHandle>,
    page: usize,
) -> serenity::builder::CreateEmbed {
    let mut titles = Vec::with_capacity(10);
    let mut artists = Vec::with_capacity(10);
    let mut durs = Vec::with_capacity(10);

    for (i, track) in queue.iter().enumerate().skip(page * 10).take(10) {
        let m = track.metadata();

        let title = format!("**{}.** {}\n", i + 1, get_title(m));
        titles.push(title);

        let artist = format!("{}\n", get_artist(m));
        artists.push(artist);

        let duration = m.duration.unwrap_or_default();
        let seconds = duration.as_secs() % 60;
        let minutes = duration.as_secs() / 60;
        durs.push(format!("`[{}:{}]`\n", minutes, seconds));
    }

    let mut e = serenity::builder::CreateEmbed::default();
    e.author(|a| a.name("Queueueueueu"));

    if let Some(track) = queue.get(1) {
        let m = track.metadata();

        e.description(format!(
            "**Up Next:** {} by {}",
            get_title(m),
            get_artist(m)
        ));
    }

    // * Change to intersperse after #79524 stablizes
    e.field(
        "# Title",
        string_or_default(titles.into_iter().collect::<String>().trim_end(), "Queue"),
        true,
    );

    e.field(
        "Artist",
        string_or_default(artists.into_iter().collect::<String>().trim_end(), "is"),
        true,
    );

    e.field(
        "Duration",
        string_or_default(durs.into_iter().collect::<String>().trim_end(), "empty"),
        true,
    );

    e.footer(|f| f.text(format!("Page {}/{}.", page + 1, queue.len() / 10 + 1)));

    e
}

fn get_title(m: &Metadata) -> &str {
    m.track
        .as_deref()
        .or_else(|| m.title.as_deref())
        .unwrap_or("Unknown Title")
}

fn get_artist(m: &Metadata) -> &str {
    m.artist
        .as_deref()
        .or_else(|| m.channel.as_deref())
        .unwrap_or("Unknown Artist")
}

fn string_or_default<'a>(s: &'a str, d: &'a str) -> &'a str {
    if s.is_empty() {
        d
    } else {
        s
    }
}
