use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Notification {
    pub(crate) id: u32,

    pub(crate) app_name: String,

    pub(crate) summary: String,

    pub(crate) body: String,

    pub(crate) icon: String,

    pub(crate) image: String,

    pub(crate) urgency: Urgency,

    // the "default" action is kept apart, it has no button of its own
    pub(crate) actions: Vec<Action>,

    pub(crate) has_default: bool,

    // stays open after an action, until it is dismissed or the sender closes it
    pub(crate) resident: bool,

    pub(crate) received: SystemTime,
}

// a button a notification asks for, like "reply" or "open"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub(crate) key: String,

    pub(crate) label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Urgency {
    Low,

    // what a notification without the hint gets
    #[default]
    Normal,

    Critical,
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

    // a picture for this one notification, like a sender's avatar: a file path, a file url, or empty
    pub fn image(&self) -> &str {
        &self.image
    }

    pub fn urgency(&self) -> Urgency {
        self.urgency
    }

    // the buttons to show, in the order the sender gave them
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    // whether clicking the notification itself does something, see Notifications::click
    pub fn has_default_action(&self) -> bool {
        self.has_default
    }

    // when it arrived, or when the sender last replaced it
    pub fn received(&self) -> SystemTime {
        self.received
    }
}

impl Action {
    // pass this to Notifications::invoke
    pub fn key(&self) -> &str {
        &self.key
    }

    // the text to show on the button
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Urgency {
    // the spec sends urgency as a byte: 0 low, 1 normal, 2 critical
    pub(crate) fn from_level(level: f64) -> Self {
        match level as u8 {
            0 => Self::Low,
            2 => Self::Critical,
            _ => Self::Normal,
        }
    }
}
