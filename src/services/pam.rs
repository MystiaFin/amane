mod config;
mod native;
mod types;

#[cfg(test)]
mod tests;

use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::Service;

pub use config::PamConfig;
pub use types::{PamError, PamMessage, PamMessageKind, PamResult};

// one authentication at a time, separate from whether it belongs to the session lock
#[derive(Default)]
pub struct Pam {
    session: Option<Arc<Session>>,
    messages: Vec<PamMessage>,
    prompt: Option<(u64, Sender<String>)>,
    result: Option<PamResult>,
    next_message: u64,
}

#[derive(Default)]
pub(super) struct Session {
    aborted: AtomicBool,
}

impl Session {
    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Release);
    }

    pub fn aborted(&self) -> bool {
        self.aborted.load(Ordering::Acquire)
    }
}

impl Service for Pam {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl Pam {
    // true until the native worker has finished, including cleanup after cancellation
    pub fn active(&self) -> bool {
        self.session.is_some()
    }

    pub fn messages(&self) -> &[PamMessage] {
        &self.messages
    }

    // only the current unanswered prompt; its id must accompany a response
    pub fn prompt(&self) -> Option<&PamMessage> {
        let (id, _) = self.prompt.as_ref()?;
        self.messages.last().filter(|message| message.id() == *id)
    }

    pub fn result(&self) -> Option<&PamResult> {
        self.result.as_ref()
    }

    /// Starts authentication without unlocking the session. Read prompts through `Pam::read()`.
    pub fn start(config: PamConfig) -> Result<(), PamError> {
        start(config, None, Arc::new(Session::default()), None)
    }

    /// Answers the prompt with this id. Stale ids and duplicate responses are rejected.
    pub fn respond(id: u64, response: &str) -> Result<(), PamError> {
        if response.contains('\0') {
            return Err(PamError::InvalidInput("response"));
        }

        let mut pam = Self::write();
        if !pam
            .prompt
            .as_ref()
            .is_some_and(|(current, _)| *current == id)
        {
            pam.quiet();
            return Err(PamError::NoPrompt);
        }
        let (_, sender) = pam.prompt.take().ok_or(PamError::NoPrompt)?;
        sender
            .send(String::from(response))
            .map_err(|_| PamError::NoPrompt)
    }

    /// Cancels the attempt and wakes a waiting prompt. Late native results are discarded.
    pub fn abort() {
        let mut pam = Self::write();
        pam.cancel();
    }

    fn cancel(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        session.abort();
        self.prompt = None;
        self.result = Some(PamResult::Aborted);
    }
}

type Completion = fn(&Arc<Session>, &PamResult);

pub(super) fn start(
    config: PamConfig,
    password: Option<String>,
    session: Arc<Session>,
    completed: Option<Completion>,
) -> Result<(), PamError> {
    let config = config.prepare()?;
    if password
        .as_ref()
        .is_some_and(|password| password.contains('\0'))
    {
        return Err(PamError::InvalidInput("response"));
    }

    {
        let mut pam = Pam::write();
        // A lock can cancel its reserved attempt before it reaches the PAM service.
        if session.aborted() {
            pam.quiet();
            return Ok(());
        }
        if pam.active() {
            pam.quiet();
            return Err(PamError::Busy);
        }
        pam.session = Some(session.clone());
        pam.messages.clear();
        pam.prompt = None;
        pam.result = None;
    }

    let current = session.clone();
    let spawned = thread::Builder::new()
        .name(String::from("amane-pam"))
        .spawn(move || {
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                native::authenticate(&config, &current, password.as_deref())
            }))
            .unwrap_or_else(|_| {
                PamResult::Error(PamError::Worker(String::from(
                    "authentication worker panicked",
                )))
            });
            finish(&current, result, completed);
        });

    if let Err(error) = spawned {
        let error = PamError::Worker(error.to_string());
        finish(&session, PamResult::Error(error.clone()), None);
        return Err(error);
    }
    Ok(())
}

fn finish(session: &Arc<Session>, result: PamResult, completed: Option<Completion>) {
    let result = {
        let mut pam = Pam::write();
        if !pam
            .session
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, session))
        {
            pam.quiet();
            return;
        }
        let result = if session.aborted() {
            PamResult::Aborted
        } else {
            result
        };
        pam.prompt = None;
        pam.session = None;
        pam.result = Some(result.clone());
        result
    };

    // never hold the PAM write guard while a lock completion takes its own write guard
    if let Some(completed) = completed {
        completed(session, &result);
    }
}

pub(super) fn abort_session(session: &Arc<Session>) {
    let mut pam = Pam::write();
    if pam
        .session
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, session))
    {
        pam.cancel();
    } else {
        // the worker may have finished just before the lock reset, but its callback must not unlock
        session.abort();
        pam.quiet();
    }
}

fn message(
    session: &Arc<Session>,
    text: String,
    kind: PamMessageKind,
    password: Option<&str>,
) -> Result<Option<String>, ()> {
    let prompt = matches!(kind, PamMessageKind::Prompt { .. });
    let (sender, receiver) = mpsc::channel();
    {
        let mut pam = Pam::write();
        if session.aborted()
            || !pam
                .session
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, session))
        {
            pam.quiet();
            return Err(());
        }
        pam.next_message = pam.next_message.wrapping_add(1);
        let id = pam.next_message;
        pam.messages.push(PamMessage { id, text, kind });
        if prompt && password.is_none() {
            pam.prompt = Some((id, sender));
        }
    }
    if !prompt {
        return Ok(None);
    }
    if let Some(password) = password {
        return Ok(Some(String::from(password)));
    }
    receiver.recv().map(Some).map_err(|_| ())
}
