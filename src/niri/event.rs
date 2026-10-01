use super::NiriWorkspace;

// the niri events amane uses, everything else niri sends is skipped
pub enum Event {
    // the full list, sent first and again whenever one is added or removed
    Workspaces(Vec<NiriWorkspace>),

    // focused is false when it only became the one shown on its monitor
    Activated { id: u64, focused: bool },

    Urgent { id: u64, urgent: bool },

    // every window and the workspace it is on, if any; sent first
    Windows(Vec<(u64, Option<u64>)>),

    // a window opened, or moved to another workspace
    WindowChanged { id: u64, workspace: Option<u64> },

    WindowClosed { id: u64 },
}
