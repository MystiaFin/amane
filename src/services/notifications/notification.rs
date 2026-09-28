use crate::Urgency;

#[derive(Debug, Clone)]
pub struct Notification {
    pub(crate) id: u32,

    pub(crate) app_name: String,

    pub(crate) summary: String,

    pub(crate) body: String,

    pub(crate) icon: String,

    pub(crate) urgency: Urgency,
}

impl Notification {
    // pass this to Notifications::dismiss
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    // the one-line title
    pub fn summary(&self) -> &str {
        &self.summary
    }

    // may be empty, and may hold simple markup like <b>
    pub fn body(&self) -> &str {
        &self.body
    }

    // an icon name like "firefox", a file path, or empty
    pub fn icon(&self) -> &str {
        &self.icon
    }

    pub fn urgency(&self) -> Urgency {
        self.urgency
    }
}
