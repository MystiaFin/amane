use super::pulse::Stream;

/*
 * an app playing sound, with all of its streams under one level;
 * electron apps all call themselves "Chromium", so the process
 * keeps two of them apart
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioApp {
    // what the app calls itself, like "Firefox"
    pub(crate) name: String,

    // a freedesktop icon name like "firefox", only when the app gives one
    pub(crate) icon: Option<String>,

    // the program's file name like "firefox", for matching it to a desktop entry
    pub(crate) binary: Option<String>,

    // the process playing the sound, only when the app says
    pub(crate) process: Option<u32>,

    // 0 to 100, the loudest of its streams
    pub(crate) volume: u8,

    // only when every stream is muted
    pub(crate) muted: bool,
}

impl AudioApp {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn binary(&self) -> Option<&str> {
        self.binary.as_deref()
    }

    pub fn process(&self) -> Option<u32> {
        self.process
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub(crate) fn plays(&self, stream: &Stream) -> bool {
        self.name == stream.app && self.process == stream.process
    }
}

// one app per name and process, in the order their first stream started
pub(crate) fn group(streams: &[Stream]) -> Vec<AudioApp> {
    let mut apps: Vec<AudioApp> = Vec::new();

    for stream in streams {
        if let Some(app) = apps.iter_mut().find(|app| app.plays(stream)) {
            app.volume = app.volume.max(stream.volume);
            app.muted &= stream.muted;

            continue;
        }

        apps.push(AudioApp {
            name: stream.app.clone(),
            icon: stream.icon.clone(),
            binary: stream.binary.clone(),
            process: stream.process,
            volume: stream.volume,
            muted: stream.muted,
        });
    }

    apps
}

#[cfg(test)]
mod tests {
    use libpulse_binding::volume::ChannelVolumes;

    use super::*;

    fn stream(app: &str, process: Option<u32>, volume: u8, muted: bool) -> Stream {
        Stream {
            index: 0,
            volumes: ChannelVolumes::default(),
            app: String::from(app),
            icon: None,
            binary: None,
            process,
            volume,
            muted,
        }
    }

    #[test]
    fn keeps_apps_with_the_same_name_apart_by_process() {
        let streams = [
            stream("Chromium", Some(10), 50, false),
            stream("Chromium", Some(20), 80, false),
            stream("Chromium", Some(10), 30, false),
        ];

        let apps = group(&streams);

        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].process(), Some(10));
        assert_eq!(apps[1].process(), Some(20));
    }

    #[test]
    fn shows_the_loudest_stream() {
        let apps = group(&[
            stream("Zen", Some(1), 40, false),
            stream("Zen", Some(1), 90, false),
        ]);

        assert_eq!(apps[0].volume(), 90);
    }

    #[test]
    fn is_muted_only_when_every_stream_is() {
        let some = group(&[
            stream("Zen", Some(1), 40, true),
            stream("Zen", Some(1), 40, false),
        ]);
        let every = group(&[
            stream("Zen", Some(1), 40, true),
            stream("Zen", Some(1), 40, true),
        ]);

        assert!(!some[0].muted());
        assert!(every[0].muted());
    }

    #[test]
    fn gives_no_apps_for_no_streams() {
        assert!(group(&[]).is_empty());
    }
}
