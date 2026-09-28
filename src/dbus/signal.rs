use crate::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    // the sender's unique name like :1.42, not a well-known name like org.mpris.MediaPlayer2.mpv
    pub(crate) sender: String,

    pub(crate) path: String,

    pub(crate) arguments: Vec<Value>,
}

impl Signal {
    pub fn sender(&self) -> &str {
        &self.sender
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }
}
