use std::sync::{Mutex, PoisonError};

use crate::Service;
use crate::changes;

// set by Clipboard::paste, taken by the event loop when it wakes
static PASTE_REQUESTED: Mutex<Option<String>> = Mutex::new(None);

// older copies fall off past this, so the history can't grow forever
const MAX_ENTRIES: usize = 100;

#[derive(Default)]
pub struct Clipboard {
    list: Vec<String>,
}

impl Service for Clipboard {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl Clipboard {
    // newest first
    pub fn list(&self) -> &[String] {
        &self.list
    }

    // a new copy goes to the top
    pub(crate) fn add(&mut self, text: String) {
        if text.is_empty() {
            return;
        }

        // copying the same text again moves it to the top instead of adding it twice
        self.list.retain(|entry| *entry != text);

        self.list.insert(0, text);

        self.list.truncate(MAX_ENTRIES);
    }

    /*
     * types this entry into the app that has focus, like from a click on
     * a history row; a second paste before the first is typed replaces it
     */
    pub fn paste(text: &str) {
        let mut requested = PASTE_REQUESTED
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        *requested = Some(String::from(text));

        drop(requested);

        changes::mark_all();
    }

    pub(crate) fn take_paste_request() -> Option<String> {
        let mut requested = PASTE_REQUESTED
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        requested.take()
    }
}
