use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PamMessageKind {
    Info,
    Error,
    Prompt { visible: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PamMessage {
    pub(super) id: u64,
    pub(super) text: String,
    pub(super) kind: PamMessageKind,
}

impl PamMessage {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn kind(&self) -> PamMessageKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PamResult {
    Success,
    Rejected(PamError),
    Aborted,
    Error(PamError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PamError {
    Busy,
    NoPrompt,
    InvalidInput(&'static str),
    UserUnavailable,
    UserChanged,
    Worker(String),
    Native { code: i32, message: String },
}

impl fmt::Display for PamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy => f.write_str("authentication is already running"),
            Self::NoPrompt => f.write_str("this prompt is not waiting for a response"),
            Self::InvalidInput(field) => write!(f, "invalid PAM {field}"),
            Self::UserUnavailable => f.write_str("USER is not set or is not valid text"),
            Self::UserChanged => f.write_str("PAM changed the authenticated user"),
            Self::Worker(message) => write!(f, "PAM worker: {message}"),
            Self::Native { code, message } => write!(f, "PAM error {code}: {message}"),
        }
    }
}

impl std::error::Error for PamError {}
