mod pulse;

use crate::Service;

#[derive(Default)]
pub struct Audio {
    // 0 to 100, for the default output
    volume: u8,

    muted: bool,
}

// event-driven, the sound server announces every volume change
impl Service for Audio {
    fn new() -> Self {
        let mut audio = Self::default();

        audio.update();

        audio
    }

    fn update(&mut self) {
        let sink = pulse::read();

        self.volume = sink.volume;
        self.muted = sink.muted;
    }

    fn listen() {
        pulse::watch(|| Self::write().update());
    }
}

impl Audio {
    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    // anything above 100 is treated as 100
    pub fn set_volume(volume: u8) {
        pulse::set_volume(volume.min(100));
    }

    pub fn toggle_mute() {
        let sink = pulse::read();

        pulse::set_muted(!sink.muted);
    }
}
