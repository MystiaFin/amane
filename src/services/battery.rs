use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::Service;

const POWER_SUPPLIES: &str = "/sys/class/power_supply";

#[derive(Default)]
pub struct Battery {
    // none on a desktop without a battery
    path: Option<PathBuf>,

    // 0 to 100
    percent: u8,

    charging: bool,

    // plugged in and not charging, usually because it's full
    full: bool,
}

// polled, the kernel only exposes the charge as files
impl Service for Battery {
    fn new() -> Self {
        let mut battery = Self {
            path: find(),
            ..Self::default()
        };

        battery.update();

        battery
    }

    fn interval() -> Duration {
        Duration::from_secs(5)
    }

    fn update(&mut self) {
        let Some(path) = &self.path else {
            return;
        };

        let capacity = read(path, "capacity");
        let status = read(path, "status");

        self.percent = capacity.parse().unwrap_or(0);

        self.charging = status == "Charging";
        self.full = status == "Full" || status == "Not charging";
    }
}

impl Battery {
    pub fn present(&self) -> bool {
        self.path.is_some()
    }

    pub fn percent(&self) -> u8 {
        self.percent
    }

    pub fn charging(&self) -> bool {
        self.charging
    }

    pub fn full(&self) -> bool {
        self.full
    }
}

// the first supply whose type is Battery, a laptop's AC adapter is skipped
fn find() -> Option<PathBuf> {
    let entries = fs::read_dir(POWER_SUPPLIES).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if read(&path, "type") == "Battery" {
            return Some(path);
        }
    }

    None
}

fn read(folder: &Path, name: &str) -> String {
    let text = fs::read_to_string(folder.join(name)).unwrap_or_default();

    String::from(text.trim())
}
