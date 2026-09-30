use std::fs;
use std::time::Duration;

use crate::Service;

const MEMINFO: &str = "/proc/meminfo";

/*
 * the kernel's count moves by a few kibibytes on every read, so only a
 * move of about 100 MiB, the finest a bar shows, counts as a change
 */
const SHOWN_STEP: u64 = 100 * 1024;

#[derive(Default)]
pub struct Memory {
    // in kibibytes, the unit the kernel reports
    total: u64,

    available: u64,
}

// polled, the kernel only exposes memory as a file
impl Service for Memory {
    fn new() -> Self {
        let mut memory = Self::default();

        memory.update();

        memory
    }

    fn interval() -> Duration {
        Duration::from_secs(2)
    }

    fn update(&mut self) -> bool {
        let before = (self.percent(), self.used_kib() / SHOWN_STEP);

        let text = fs::read_to_string(MEMINFO).unwrap_or_default();

        self.total = field(&text, "MemTotal:");
        self.available = field(&text, "MemAvailable:");

        (self.percent(), self.used_kib() / SHOWN_STEP) != before
    }
}

impl Memory {
    pub fn total_kib(&self) -> u64 {
        self.total
    }

    // what programs hold; cache the kernel can free right away doesn't count
    pub fn used_kib(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    // 0 to 100
    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }

        (self.used_kib() * 100 / self.total) as u8
    }
}

// lines look like "MemTotal:       16318508 kB"
fn field(text: &str, name: &str) -> u64 {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(name) else {
            continue;
        };

        let number = rest.split_whitespace().next().unwrap_or("0");

        return number.parse().unwrap_or(0);
    }

    0
}
