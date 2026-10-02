mod action;
mod image;
mod notification;
mod reason;
mod server;
mod urgency;

pub use action::Action;
pub use notification::Notification;
pub use urgency::Urgency;

use crate::Service;

use reason::Reason;

pub struct Notifications {
    list: Vec<Notification>,

    // the spec keeps 0 for "no notification", so ids start at 1
    next_id: u32,

    running: bool,
}

/*
 * amane is the notification server here: programs send
 * their notifications to it, instead of amane asking anyone
 */
impl Service for Notifications {
    fn new() -> Self {
        Self {
            list: Vec::new(),
            next_id: 1,
            running: false,
        }
    }

    fn listen() {
        server::run();
    }
}

impl Notifications {
    // oldest first
    pub fn list(&self) -> &[Notification] {
        &self.list
    }

    // false when another daemon like mako or dunst already owns the name
    pub fn running(&self) -> bool {
        self.running
    }

    pub fn dismiss(id: u32) {
        Self::write().close(id, Reason::Dismissed);
    }

    pub fn clear() {
        let mut notifications = Self::write();

        let mut ids = Vec::new();

        for notification in &notifications.list {
            ids.push(notification.id);
        }

        for id in ids {
            notifications.close(id, Reason::Dismissed);
        }
    }

    // a button was pressed, pass the key from notification.actions()
    pub fn invoke(id: u32, key: &str) {
        Self::write().run_action(id, key);
    }

    // the notification itself was clicked: its default action, or a dismiss when it has none
    pub fn click(id: u32) {
        let mut notifications = Self::write();

        let has_default = notifications
            .list
            .iter()
            .any(|notification| notification.id == id && notification.has_default);

        if has_default {
            notifications.run_action(id, server::DEFAULT_ACTION);
        } else {
            notifications.close(id, Reason::Dismissed);
        }
    }

    // the spec closes a notification once an action ran, unless it asked to stay
    fn run_action(&mut self, id: u32, key: &str) {
        let Some(notification) = self
            .list
            .iter()
            .find(|notification| notification.id == id)
        else {
            return;
        };

        let resident = notification.resident;

        server::announce_action(id, key);

        if !resident {
            self.close(id, Reason::Dismissed);
        }
    }

    // a sender may replace its own earlier notification, which keeps its id and place
    fn add(&mut self, mut notification: Notification, replaces_id: u32) -> u32 {
        let replaced = self.list.iter_mut().find(|old| old.id == replaces_id);

        if let Some(old) = replaced {
            notification.id = replaces_id;

            image::remove(&old.image);

            *old = notification;

            return replaces_id;
        }

        let id = self.next_id;

        self.next_id += 1;

        notification.id = id;

        self.list.push(notification);

        id
    }

    fn close(&mut self, id: u32, reason: Reason) {
        let Some(index) = self
            .list
            .iter()
            .position(|notification| notification.id == id)
        else {
            return;
        };

        let closed = self.list.remove(index);

        image::remove(&closed.image);

        server::announce_closed(id, reason);
    }
}
