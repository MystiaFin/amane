mod agent;
mod native;
mod state;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::env;
use std::ffi::CString;

use crate::Service;
use agent::Command;

/// A Polkit authentication agent for the shell's login session.
/// Reading it does not register an agent. Call [`Polkit::start`] from startup or an input handler.
#[derive(Clone)]
pub struct Polkit {
    enabled: bool,
    running: bool,
    path: String,
    error: Option<PolkitError>,
    flow: Option<PolkitFlow>,
    completion: Option<PolkitCompletion>,
}

/// Registration settings. The session is resolved from the process through logind.
#[derive(Clone, Debug)]
pub struct PolkitConfig {
    path: String,
    locale: String,
}

impl Default for PolkitConfig {
    fn default() -> Self {
        Self {
            path: "/org/amane/Polkit".into(),
            locale: ["LC_ALL", "LC_MESSAGES", "LANG"]
                .into_iter()
                .filter_map(|key| env::var(key).ok())
                .find(|value| !value.is_empty())
                .unwrap_or_else(|| "C".into()),
        }
    }
}

impl PolkitConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a different D-Bus object path for the agent.
    pub fn path(mut self, path: &str) -> Self {
        self.path = path.into();
        self
    }

    /// Request localized authentication messages in this locale.
    pub fn locale(mut self, locale: &str) -> Self {
        self.locale = locale.into();
        self
    }

    fn validate(&self) -> Result<(), PolkitError> {
        zbus::zvariant::ObjectPath::try_from(self.path.as_str())
            .map_err(|_| PolkitError::InvalidConfig("invalid D-Bus object path".into()))?;
        CString::new(self.locale.as_str())
            .map_err(|_| PolkitError::InvalidConfig("locale contains a NUL byte".into()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PolkitError {
    #[error("invalid Polkit configuration: {0}")]
    InvalidConfig(String),
    #[error("Polkit registration failed: {0}")]
    Registration(String),
    #[error("Polkit connection lost: {0}")]
    Disconnected(String),
    #[error("Polkit authentication failed: {0}")]
    Authentication(String),
    #[error("this authentication request is no longer active")]
    Inactive,
    #[error("this prompt has already been answered or replaced")]
    StalePrompt,
    #[error("the selected identity is not offered by this request")]
    InvalidIdentity,
    #[error("authentication has not failed or an attempt is already in progress")]
    NotRetryable,
    #[error("authentication responses cannot contain NUL, CR, or LF")]
    InvalidResponse,
    #[error("the Polkit service is unavailable")]
    Unavailable,
}

/// A user Polkit permits to authenticate this request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolkitIdentity {
    pub(super) uid: u32,
    pub(super) name: String,
}

impl PolkitIdentity {
    pub fn uid(&self) -> u32 {
        self.uid
    }
    /// The account name, or its numeric UID if the account cannot be resolved.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// One input prompt. Its ID changes for every prompt, including retries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolkitPrompt {
    id: u64,
    text: String,
    visible: bool,
}

impl PolkitPrompt {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
}

/// Supplementary information or an error from the authentication conversation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolkitMessage {
    text: String,
    error: bool,
}

impl PolkitMessage {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn is_error(&self) -> bool {
        self.error
    }
}

/// The current authorization request and its authentication conversation.
#[derive(Clone, Debug)]
pub struct PolkitFlow {
    id: u64,
    action_id: String,
    message: String,
    icon: String,
    details: BTreeMap<String, String>,
    identities: Vec<PolkitIdentity>,
    selected: usize,
    prompt: Option<PolkitPrompt>,
    supplementary: Option<PolkitMessage>,
    failed: bool,
    retry_allowed: bool,
}

impl PolkitFlow {
    fn new(
        id: u64,
        action_id: String,
        message: String,
        icon: String,
        details: BTreeMap<String, String>,
        identities: Vec<PolkitIdentity>,
    ) -> Self {
        Self {
            id,
            action_id,
            message,
            icon,
            details,
            identities,
            selected: 0,
            prompt: None,
            supplementary: None,
            failed: false,
            retry_allowed: false,
        }
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn action_id(&self) -> &str {
        &self.action_id
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn icon(&self) -> &str {
        &self.icon
    }
    pub fn details(&self) -> &BTreeMap<String, String> {
        &self.details
    }
    pub fn identities(&self) -> &[PolkitIdentity] {
        &self.identities
    }
    pub fn selected_identity(&self) -> usize {
        self.selected
    }
    pub fn prompt(&self) -> Option<&PolkitPrompt> {
        self.prompt.as_ref()
    }
    pub fn supplementary(&self) -> Option<&PolkitMessage> {
        self.supplementary.as_ref()
    }
    /// Whether an attempt has failed during this request, including before a retry.
    pub fn failed(&self) -> bool {
        self.failed
    }
    /// Whether a rejected attempt is waiting for the user to retry.
    pub fn can_retry(&self) -> bool {
        self.retry_allowed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolkitResult {
    Success,
    Cancelled,
    Error(PolkitError),
}

/// The most recently completed displayed request. Queued cancellations do not replace it.
#[derive(Clone, Debug)]
pub struct PolkitCompletion {
    flow: PolkitFlow,
    result: PolkitResult,
}

impl PolkitCompletion {
    pub fn flow(&self) -> &PolkitFlow {
        &self.flow
    }
    pub fn result(&self) -> &PolkitResult {
        &self.result
    }
}

impl Service for Polkit {
    fn new() -> Self {
        Self::new_state()
    }
    fn listen() {
        agent::run();
    }
}

impl Polkit {
    fn new_state() -> Self {
        Self {
            enabled: false,
            running: false,
            path: PolkitConfig::default().path,
            error: None,
            flow: None,
            completion: None,
        }
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn running(&self) -> bool {
        self.running
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn error(&self) -> Option<&PolkitError> {
        self.error.as_ref()
    }
    pub fn flow(&self) -> Option<&PolkitFlow> {
        self.flow.as_ref()
    }
    pub fn completion(&self) -> Option<&PolkitCompletion> {
        self.completion.as_ref()
    }

    /// Queue registration. Read `running()` and `error()` for its asynchronous result.
    pub fn start(config: PolkitConfig) -> Result<(), PolkitError> {
        config.validate()?;
        agent::send(Command::Start(config))
    }
    /// Cancel pending requests and unregister the agent.
    pub fn stop() -> Result<(), PolkitError> {
        agent::send(Command::Stop)
    }

    /// Change to an identity offered by this request, restarting its conversation.
    pub fn select_identity(request: u64, index: usize) -> Result<(), PolkitError> {
        let state = Self::read();
        let flow = state
            .flow
            .as_ref()
            .filter(|flow| flow.id == request)
            .ok_or(PolkitError::Inactive)?;
        if index >= flow.identities.len() {
            return Err(PolkitError::InvalidIdentity);
        }
        agent::send(Command::Select(request, index))
    }
    /// Answer exactly the prompt shown by the view. Stale replies are never reused.
    pub fn respond(request: u64, prompt: u64, response: &str) -> Result<(), PolkitError> {
        let response = native::Secret::new(response)?;
        let state = Self::read();
        let flow = state
            .flow
            .as_ref()
            .filter(|flow| flow.id == request)
            .ok_or(PolkitError::Inactive)?;
        if flow.prompt.as_ref().is_none_or(|value| value.id != prompt) {
            return Err(PolkitError::StalePrompt);
        }
        agent::send(Command::Respond(request, prompt, response))
    }
    /// Retry a failed attempt with the current identity.
    pub fn retry(request: u64) -> Result<(), PolkitError> {
        let state = Self::read();
        let flow = state
            .flow
            .as_ref()
            .filter(|flow| flow.id == request)
            .ok_or(PolkitError::Inactive)?;
        if !flow.can_retry() {
            return Err(PolkitError::NotRetryable);
        }
        agent::send(Command::Retry(request))
    }
    /// Dismiss this request, including its pending D-Bus call.
    pub fn cancel(request: u64) -> Result<(), PolkitError> {
        agent::send(Command::Cancel(request))
    }
}
