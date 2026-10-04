use crate::compositor;
use crate::services::worker;
use crate::{Service, Workspace};

pub struct Workspaces {
    list: Vec<Workspace>,
}

impl Service for Workspaces {
    fn new() -> Self {
        Self { list: Vec::new() }
    }

    // on an unsupported compositor the list stays empty
    fn listen() {
        compositor::listen(|mut list| {
            // compositors send them in no particular order
            list.sort_by(|first, second| {
                let first_place = (&first.output, first.index);
                let second_place = (&second.output, second.index);

                first_place.cmp(&second_place)
            });

            let mut workspaces = Self::write();

            // a window's title changing is a compositor event too, and changes no workspace
            if workspaces.list == list {
                workspaces.quiet();

                return;
            }

            workspaces.list = list;
        });
    }
}

impl Workspaces {
    // sorted by monitor, then by position on it
    pub fn list(&self) -> &[Workspace] {
        &self.list
    }

    pub fn focus(id: i64) {
        worker::run(move || compositor::focus_workspace(id));
    }
}
