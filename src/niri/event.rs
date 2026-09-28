use crate::Workspace;

// the niri events amane uses, everything else niri sends is skipped
pub enum Event {
    // the full list, sent first and again whenever one is added or removed
    Workspaces(Vec<Workspace>),

    // focused is false when it only became the one shown on its monitor
    Activated { id: u64, focused: bool },

    Urgent { id: u64, urgent: bool },
}
