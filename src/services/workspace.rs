#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub(crate) id: u64,

    // position on its monitor, starting at 1
    pub(crate) index: u32,

    pub(crate) name: Option<String>,
    pub(crate) output: Option<String>,

    // shown on its monitor, even when another monitor has focus
    pub(crate) active: bool,

    pub(crate) focused: bool,
    pub(crate) urgent: bool,

    // how many windows are on it
    pub(crate) windows: u32,
}

impl Workspace {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn urgent(&self) -> bool {
        self.urgent
    }

    // 0 for an empty workspace, one that only shows the desktop
    pub fn windows(&self) -> u32 {
        self.windows
    }
}
