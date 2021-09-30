use serenity::{model::channel::Message, Result as SerenityResult};
use songbird::{input::Metadata, tracks::TrackHandle};
use std::time::Duration;

pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

/// `split_duration` splits a [`Duration`] into a (minutes, seconds) tuple
const fn split_duration(d: Duration) -> (u64, u64) {
    (d.as_secs() / 60, d.as_secs() % 60)
}

pub fn generate_embed(
    m: &Metadata,
    pos: Duration,
    m2: Option<&Metadata>,
) -> serenity::builder::CreateEmbed {
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

    let (curr_min, curr_sec) = split_duration(pos);
    let (max_min, max_sec) = split_duration(m.duration.unwrap_or_default());
    let progress = format!(
        "**Current Time:** {}:{:02} / {}:{:02}",
        curr_min, curr_sec, max_min, max_sec
    );

    let up_next = m2
        .map(|m2| format!("**Up Next:** {} by {}", get_title(m2), get_artist(m2)))
        .unwrap_or_default();

    e.description(&[progress, up_next].join("\n"));
    e.timestamp(&chrono::Utc::now());

    e
}

pub fn generate_queue_embed(queue: &[TrackHandle], page: usize) -> serenity::builder::CreateEmbed {
    let mut titles = Vec::with_capacity(10);
    let mut artists = Vec::with_capacity(10);
    let mut durs = Vec::with_capacity(10);
    let total_duration = queue.iter().fold(Duration::default(), |a, b| {
        a + b.metadata().duration.unwrap_or_default()
    });

    for (i, track) in queue.iter().enumerate().skip(page * 10).take(10) {
        let m = track.metadata();

        let title = format!("**{}.** {}\n", i + 1, get_title(m));
        titles.push(title);

        let artist = format!("{}\n", get_artist(m));
        artists.push(artist);

        let duration = m.duration.unwrap_or_default();
        let seconds = duration.as_secs() % 60;
        let minutes = duration.as_secs() / 60;

        durs.push(format!("[{}:{:02}]\n", minutes, seconds));
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

    e.footer(|f| {
        let seconds = total_duration.as_secs() % 60;
        let minutes = total_duration.as_secs() / 60;

        f.text(format!(
            "Page {}/{} | Total Duration: {:02}:{:02}",
            page + 1,
            (queue.len() / 10 + 1),
            minutes,
            seconds,
        ))
    });

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

const fn string_or_default<'a>(s: &'a str, d: &'a str) -> &'a str {
    if s.is_empty() {
        d
    } else {
        s
    }
}
