use std::collections::HashMap;

use crate::niri::{self, Event, NiriWorkspace};
use crate::services::worker;
use crate::{Service, Workspace};

pub struct Workspaces {
    list: Vec<Workspace>,

    // every window by its id, and the workspace it is on
    windows: HashMap<u64, Option<u64>>,
}

impl Service for Workspaces {
    fn new() -> Self {
        Self {
            list: Vec::new(),
            windows: HashMap::new(),
        }
    }

    // without niri the list stays empty
    fn listen() {
        let Some(events) = niri::events() else {
            return;
        };

        for event in events {
            let mut workspaces = Self::write();

            let before = workspaces.list.clone();

            workspaces.apply(event);

            // a window's title changing is a window event too, and changes no workspace
            if workspaces.list == before {
                workspaces.quiet();
            }
        }
    }
}

impl Workspaces {
    // sorted by monitor, then by position on it
    pub fn list(&self) -> &[Workspace] {
        &self.list
    }

    pub fn focus(id: u64) {
        worker::run(move || niri::focus_workspace(id));
    }

    fn apply(&mut self, event: Event) {
        match event {
            Event::Workspaces(list) => self.replace(list),
            Event::Activated { id, focused } => self.activate(id, focused),
            Event::Urgent { id, urgent } => self.mark_urgent(id, urgent),

            Event::Windows(windows) => {
                self.windows = windows.into_iter().collect();
            }

            Event::WindowChanged { id, workspace } => {
                self.windows.insert(id, workspace);
            }

            Event::WindowClosed { id } => {
                self.windows.remove(&id);
            }
        }

        self.count_windows();
    }

    fn replace(&mut self, reported: Vec<NiriWorkspace>) {
        let mut list = Vec::new();

        for workspace in reported {
            list.push(Workspace {
                id: workspace.id,
                index: workspace.index,

                name: workspace.name,
                output: workspace.output,

                active: workspace.active,
                focused: workspace.focused,
                urgent: workspace.urgent,

                // counted from the window events, after every event
                windows: 0,
            });
        }

        // niri sends them in no particular order
        list.sort_by(|first, second| {
            let first_place = (&first.output, first.index);
            let second_place = (&second.output, second.index);

            first_place.cmp(&second_place)
        });

        self.list = list;
    }

    fn activate(&mut self, id: u64, focused: bool) {
        let Some(activated) = self.list.iter().find(|workspace| workspace.id == id) else {
            return;
        };

        let output = activated.output.clone();

        // only one workspace is shown per monitor, and only one has focus overall
        for workspace in &mut self.list {
            if workspace.output == output {
                workspace.active = workspace.id == id;
            }

            if focused {
                workspace.focused = workspace.id == id;
            }
        }
    }

    fn mark_urgent(&mut self, id: u64, urgent: bool) {
        for workspace in &mut self.list {
            if workspace.id == id {
                workspace.urgent = urgent;
            }
        }
    }

    // a new workspace list starts at 0 windows each, so the counts are made again after any event
    fn count_windows(&mut self) {
        for workspace in &mut self.list {
            let mut count = 0;

            for on in self.windows.values() {
                if *on == Some(workspace.id) {
                    count += 1;
                }
            }

            workspace.windows = count;
        }
    }
}
