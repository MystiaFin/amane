mod data_dirs;
mod desktop_app;
mod exec;
mod icons;
mod parse;
mod walk;

use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use crate::Service;

pub use desktop_app::DesktopApp;

// the programs a launcher can show, read from the .desktop files in the xdg data dirs
#[derive(Default)]
pub struct Apps {
    list: Vec<DesktopApp>,
}

// polled, so programs installed while the shell runs show up
impl Service for Apps {
    // empty at first: the scan walks every icon theme, which takes seconds
    fn new() -> Self {
        Self::default()
    }

    fn interval() -> Duration {
        Duration::from_secs(30)
    }

    // scanned outside the lock, so a view reading the list never waits on the disk
    fn listen() {
        loop {
            let list = scan();

            let mut apps = Self::write();

            if apps.list == list {
                apps.quiet();
            } else {
                apps.list = list;
            }

            drop(apps);

            thread::sleep(Self::interval());
        }
    }
}

impl Apps {
    // sorted by name, without the ones marked hidden or not to be shown
    pub fn list(&self) -> &[DesktopApp] {
        &self.list
    }
}

fn scan() -> Vec<DesktopApp> {
    let icons = icons::index();

    // a file in an earlier dir overrides one with the same id in a later dir
    let mut seen = HashSet::new();

    let mut apps = Vec::new();

    for dir in data_dirs::find() {
        let applications = dir.join("applications");

        for path in walk::files(&applications) {
            let Some(id) = parse::id(&applications, &path) else {
                continue;
            };

            if !seen.insert(id) {
                continue;
            }

            let Some(app) = parse::read(&path, &icons) else {
                continue;
            };

            apps.push(app);
        }
    }

    apps.sort_by_key(|app| app.name().to_lowercase());

    apps
}
