use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};

// a rebuild takes seconds, so checking twice a second is quick enough
const INTERVAL: Duration = Duration::from_millis(500);

pub fn wait_for_change(folder: &Path, before: Option<SystemTime>) {
    loop {
        if snapshot(folder) != before {
            return;
        }

        thread::sleep(INTERVAL);
    }
}

// the newest time in the folder, which moves forward whenever anything in it is saved
pub fn snapshot(folder: &Path) -> Option<SystemTime> {
    // folders count too, since adding or removing a file only changes its folder's time
    let mut latest = modified(folder);

    let Ok(entries) = fs::read_dir(folder) else {
        return latest;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let time = if path.is_dir() {
            snapshot(&path)
        } else {
            modified(&path)
        };

        latest = Option::max(latest, time);
    }

    latest
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}
