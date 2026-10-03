use std::ffi::OsString;
use std::path::Path;

use inotify::{Inotify, WatchMask};

pub struct FileChanges {
    inotify: Inotify,

    name: OsString,

    buffer: [u8; 4096],
}

/*
 * editors save by writing a new file and renaming it over the old one,
 * which ends a watch on the file itself, so the folder is watched instead
 */
pub fn watch_file(path: &str) -> FileChanges {
    let path = Path::new(path);

    let name = path
        .file_name()
        .expect("failed to watch file: no file name");

    let folder = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    let inotify = Inotify::init().expect("failed to start watching files");

    let mask = WatchMask::MODIFY | WatchMask::MOVED_TO | WatchMask::CREATE;

    inotify
        .watches()
        .add(folder, mask)
        .expect("failed to watch file");

    FileChanges {
        inotify,

        name: name.to_os_string(),

        buffer: [0; 4096],
    }
}

// waits for the next change, so call it from a service thread, not from view()
impl Iterator for FileChanges {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        loop {
            let events = self.inotify.read_events_blocking(&mut self.buffer).ok()?;

            for event in events {
                if event.name == Some(self.name.as_os_str()) {
                    return Some(());
                }
            }
        }
    }
}
