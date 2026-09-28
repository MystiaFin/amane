mod player;

use std::time::Duration;

use crate::{Service, Value};

#[derive(Default)]
pub struct Media {
    title: String,
    artist: String,
    art_url: String,

    playing: bool,

    position: Duration,
    length: Duration,
}

/*
 * polled, because players don't announce
 * the position while a track plays
 */
impl Service for Media {
    fn new() -> Self {
        let mut media = Self::default();

        media.update();

        media
    }

    fn update(&mut self) {
        let Some(player) = player::find() else {
            *self = Self::default();

            return;
        };

        let properties = player::properties(&player);
        let metadata = properties.get("Metadata");

        self.title = String::from(metadata.get("xesam:title").text());
        self.artist = first_artist(metadata.get("xesam:artist"));
        self.art_url = String::from(metadata.get("mpris:artUrl").text());

        self.playing = properties.get("PlaybackStatus").text() == "Playing";

        self.position = micros(properties.get("Position"));
        self.length = micros(metadata.get("mpris:length"));
    }
}

impl Media {
    // empty when no player is open
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }

    // usually a file:// or https:// link to the cover
    pub fn art_url(&self) -> &str {
        &self.art_url
    }

    pub fn playing(&self) -> bool {
        self.playing
    }

    pub fn position(&self) -> Duration {
        self.position
    }

    pub fn length(&self) -> Duration {
        self.length
    }

    pub fn play_pause() {
        player::send("PlayPause");
    }

    pub fn next() {
        player::send("Next");
    }

    pub fn previous() {
        player::send("Previous");
    }
}

// mpris gives a list of artists, a bar only has room for one
fn first_artist(artists: &Value) -> String {
    let Some(first) = artists.list().first() else {
        return String::new();
    };

    String::from(first.text())
}

// mpris counts time in microseconds, and a missing value reads as 0
fn micros(value: &Value) -> Duration {
    let micros = value.number().max(0.0) as u64;

    Duration::from_micros(micros)
}
