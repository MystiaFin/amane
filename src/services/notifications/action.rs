// a button a notification asks for, like "reply" or "open"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub(crate) key: String,

    pub(crate) label: String,
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
