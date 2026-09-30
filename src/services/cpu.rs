use std::fs;
use std::time::Duration;

use crate::Service;

const STAT: &str = "/proc/stat";

#[derive(Default)]
pub struct Cpu {
    // 0 to 100, across all cores
    percent: u8,

    // the counters from the last read, usage is the difference to the next one
    busy: u64,
    total: u64,
}

// polled, usage only exists as the change between two reads
impl Service for Cpu {
    fn new() -> Self {
        let mut cpu = Self::default();

        cpu.update();

        cpu
    }

    fn interval() -> Duration {
        Duration::from_secs(2)
    }

    fn update(&mut self) -> bool {
        let before = self.percent;

        let (busy, total) = read();

        let busy_since = busy.saturating_sub(self.busy);
        let total_since = total.saturating_sub(self.total);

        if total_since > 0 {
            self.percent = (busy_since * 100 / total_since) as u8;
        }

        self.busy = busy;
        self.total = total;

        // the counters always move, only the percentage is shown
        self.percent != before
    }
}

impl Cpu {
    pub fn percent(&self) -> u8 {
        self.percent
    }
}

/*
 * the first line adds up every core: "cpu user nice system idle
 * iowait irq softirq steal ...", counted in ticks since boot;
 * idle and iowait are the time spent doing nothing
 */
fn read() -> (u64, u64) {
    let text = fs::read_to_string(STAT).unwrap_or_default();

    let line = text.lines().next().unwrap_or_default();

    let mut total = 0;
    let mut idle = 0;

    for (index, field) in line.split_whitespace().skip(1).enumerate() {
        let ticks: u64 = field.parse().unwrap_or(0);

        // guest time is already counted inside user and nice
        if index >= 8 {
            break;
        }

        if index == 3 || index == 4 {
            idle += ticks;
        }

        total += ticks;
    }

    (total - idle, total)
}
