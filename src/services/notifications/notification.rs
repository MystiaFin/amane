use std::time::SystemTime;

use crate::Urgency;

use super::Action;

#[derive(Debug, Clone)]
pub struct Notification {
    pub(crate) id: u32,

    pub(crate) app_name: String,

    pub(crate) summary: String,

    pub(crate) body: String,

    pub(crate) icon: String,

    pub(crate) urgency: Urgency,

    // the "default" action is kept apart, it has no button of its own
    pub(crate) actions: Vec<Action>,

    pub(crate) has_default: bool,

    // stays open after an action, until it is dismissed or the sender closes it
    pub(crate) resident: bool,

    pub(crate) received: SystemTime,
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
