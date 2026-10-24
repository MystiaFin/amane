use crate::Service;

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
}
