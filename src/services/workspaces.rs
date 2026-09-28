use crate::niri::{self, Event};
use crate::{Service, Workspace};

pub struct Workspaces {
    list: Vec<Workspace>,
}

impl Service for Workspaces {
    fn new() -> Self {
        Self { list: Vec::new() }
    }

    fn listen() {
        for event in niri::events() {
            Self::write().apply(event);
        }
    }
}

impl Workspaces {
    // sorted by monitor, then by position on it
    pub fn list(&self) -> &[Workspace] {
        &self.list
    }

    pub fn focus(id: u64) {
        niri::focus_workspace(id);
    }

    fn apply(&mut self, event: Event) {
        match event {
            Event::Workspaces(list) => self.replace(list),
            Event::Activated { id, focused } => self.activate(id, focused),
            Event::Urgent { id, urgent } => self.mark_urgent(id, urgent),
        }
    }

    fn replace(&mut self, mut list: Vec<Workspace>) {
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
}
