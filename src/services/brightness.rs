use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::services::worker;
use crate::{Argument, Bus, Service};

const BACKLIGHTS: &str = "/sys/class/backlight";

const LOGIND: &str = "org.freedesktop.login1";
const SESSION_PATH: &str = "/org/freedesktop/login1/session/auto";
const SESSION: &str = "org.freedesktop.login1.Session";

#[derive(Default)]
pub struct Brightness {
    // none on a desktop monitor without a backlight
    path: Option<PathBuf>,

    // 0 to 100
    percent: u8,
}

/*
 * polled, because the kernel doesn't announce a change to the
 * backlight file; half a second keeps brightness keys feeling live
 */
impl Service for Brightness {
    fn new() -> Self {
        let mut brightness = Self {
            path: find(),
            ..Self::default()
        };

        brightness.update();

        brightness
    }

    fn interval() -> Duration {
        Duration::from_millis(500)
    }

    fn update(&mut self) -> bool {
        let Some(path) = &self.path else {
            return false;
        };

        let current = read(path, "brightness");
        let highest = read(path, "max_brightness");

        if highest == 0 {
            return false;
        }

        let before = self.percent;

        self.percent = (current * 100 / highest) as u8;

        self.percent != before
    }
}

impl Brightness {
    pub fn present(&self) -> bool {
        self.path.is_some()
    }

    pub fn percent(&self) -> u8 {
        self.percent
    }

    /*
     * the backlight file belongs to root, so logind writes it for us;
     * it allows that for whoever sits at the machine, with no password
     */
    pub fn set(percent: u8) {
        worker::run(move || write_level(percent));
    }
}

fn write_level(percent: u8) {
    let Some(path) = find() else {
        return;
    };

    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return;
    };

    let highest = read(&path, "max_brightness");

    // never 0, a black screen is hard to undo without seeing it
    let percent = u64::from(percent.clamp(1, 100));

    let level = (highest * percent / 100) as u32;

    let arguments = [
        Argument::from("backlight"),
        Argument::from(name),
        Argument::from(level),
    ];

    Bus::system().call(LOGIND, SESSION_PATH, SESSION, "SetBrightness", &arguments);

    Brightness::write().update();
}

// the first backlight the kernel lists, a laptop usually has exactly one
fn find() -> Option<PathBuf> {
    let mut entries = fs::read_dir(BACKLIGHTS).ok()?.flatten();

    let first = entries.next()?;

    Some(first.path())
}

fn read(folder: &Path, name: &str) -> u64 {
    let text = fs::read_to_string(folder.join(name)).unwrap_or_default();

    text.trim().parse().unwrap_or(0)
}
