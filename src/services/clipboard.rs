use crate::Service;

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
}
