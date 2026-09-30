use std::time::Duration;

use crate::Value;

use super::mpris;

// one open player, like spotify, mpv or a browser tab
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPlayer {
    // the bus name like org.mpris.MediaPlayer2.spotify, which the controls talk to
    pub(crate) name: String,

    // what the player calls itself, like "Spotify"
    pub(crate) identity: String,

    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) art_url: String,

    pub(crate) playing: bool,

    pub(crate) position: Duration,
    pub(crate) length: Duration,
}

impl MediaPlayer {
    pub(crate) fn read(name: &str) -> Self {
        let properties = mpris::properties(name);
        let metadata = properties.get("Metadata");

        Self {
            name: String::from(name),
            identity: mpris::identity(name),

            title: String::from(metadata.get("xesam:title").text()),
            artist: first_artist(metadata.get("xesam:artist")),
            art_url: String::from(metadata.get("mpris:artUrl").text()),

            playing: properties.get("PlaybackStatus").text() == "Playing",

            position: micros(properties.get("Position")),
            length: micros(metadata.get("mpris:length")),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

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

    pub fn play_pause(&self) {
        mpris::send(&self.name, "PlayPause");
    }

    pub fn next(&self) {
        mpris::send(&self.name, "Next");
    }

    pub fn previous(&self) {
        mpris::send(&self.name, "Previous");
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
