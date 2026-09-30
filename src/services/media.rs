mod mpris;
mod player;

use std::time::Duration;

use crate::Service;

pub use player::MediaPlayer;

#[derive(Default)]
pub struct Media {
    players: Vec<MediaPlayer>,

    // the player the single-player methods show and control
    active: Option<usize>,
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

    // a paused player reads the same every poll; a playing one moves its position
    fn update(&mut self) -> bool {
        let before = self.active().map(|player| player.name.clone());

        let mut players = Vec::new();

        for name in mpris::names() {
            players.push(MediaPlayer::read(&name));
        }

        let mut fresh = Self {
            players,
            active: None,
        };

        fresh.active = fresh.choose_active(before.as_deref());

        if fresh.players == self.players && fresh.active == self.active {
            return false;
        }

        *self = fresh;

        true
    }
}

impl Media {
    // every open player, in the same order on every poll
    pub fn players(&self) -> &[MediaPlayer] {
        &self.players
    }

    // the one playing, or the one shown last, or the first; none when no player is open
    pub fn active(&self) -> Option<&MediaPlayer> {
        self.players.get(self.active?)
    }

    // empty when no player is open
    pub fn title(&self) -> &str {
        self.active().map_or("", MediaPlayer::title)
    }

    pub fn artist(&self) -> &str {
        self.active().map_or("", MediaPlayer::artist)
    }

    // usually a file:// or https:// link to the cover
    pub fn art_url(&self) -> &str {
        self.active().map_or("", MediaPlayer::art_url)
    }

    pub fn playing(&self) -> bool {
        self.active().is_some_and(MediaPlayer::playing)
    }

    pub fn position(&self) -> Duration {
        self.active().map_or(Duration::ZERO, MediaPlayer::position)
    }

    pub fn length(&self) -> Duration {
        self.active().map_or(Duration::ZERO, MediaPlayer::length)
    }

    pub fn play_pause() {
        send_to_active("PlayPause");
    }

    pub fn next() {
        send_to_active("Next");
    }

    pub fn previous() {
        send_to_active("Previous");
    }

    // a paused player stays active, so pausing doesn't make the bar jump to another one
    fn choose_active(&self, before: Option<&str>) -> Option<usize> {
        if let Some(playing) = self.players.iter().position(MediaPlayer::playing) {
            return Some(playing);
        }

        if let Some(before) = before {
            if let Some(kept) = self.players.iter().position(|player| player.name == before) {
                return Some(kept);
            }
        }

        if self.players.is_empty() {
            return None;
        }

        Some(0)
    }
}

// the name is copied out, so the lock is let go before waiting on the player
fn send_to_active(method: &str) {
    let name = Media::read().active().map(|player| player.name.clone());

    // with no player open there is nothing to control
    let Some(name) = name else {
        return;
    };

    mpris::send(&name, method);
}
