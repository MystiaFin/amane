use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::pam::{self, Session};
use crate::changes;
use crate::{PamConfig, PamError, PamResult, Service};

// set by Lock::start, taken by the event loop when it wakes
static REQUESTED: AtomicBool = AtomicBool::new(false);

// the pam service file in /etc/pam.d that checks the password
const PAM_SERVICE: &str = "login";

// what the lock screen shows while App::lock holds the session
#[derive(Default)]
pub struct Lock {
    checking: bool,

    // the last authentication attempt was rejected or failed
    failed: bool,

    // the backend ends the lock once this is set, only pam sets it
    unlocked: bool,
    attempt: Option<Arc<Session>>,
}

// changes through authentication attempts rather than polling
impl Service for Lock {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl Lock {
    // an authentication attempt is in progress
    pub fn checking(&self) -> bool {
        self.checking
    }

    pub fn failed(&self) -> bool {
        self.failed
    }

    pub(crate) fn unlocked(&self) -> bool {
        self.unlocked
    }

    /*
     * locks the session with the view given to App::lock, from a button,
     * an ipc call or a thread; does nothing while already locked
     */
    pub fn start() {
        REQUESTED.store(true, Ordering::Relaxed);

        changes::mark_all();
    }

    pub(crate) fn take_request() -> bool {
        REQUESTED.swap(false, Ordering::Relaxed)
    }

    // a new lock starts clean, not with the last one's unlocked or failed state
    pub(crate) fn reset() {
        let attempt = {
            let mut lock = Self::write();
            let attempt = lock.attempt.take();
            if let Some(attempt) = &attempt {
                attempt.abort();
            }
            *lock = Self::default();
            attempt
        };
        if let Some(attempt) = attempt {
            pam::abort_session(&attempt);
        }
    }

    /*
     * pam can take seconds to answer, so it runs on its own thread
     * and the screen unlocks only if it accepts the password
     */
    pub fn unlock(password: &str) {
        let _ = Self::begin(PamConfig::new(PAM_SERVICE), Some(String::from(password)));
    }

    /// Starts interactive unlocking for the current user. Use `Pam` to read and answer prompts.
    pub fn authenticate(config: PamConfig) -> Result<(), PamError> {
        Self::begin(config, None)
    }

    /// Cancels this lock's authentication attempt and keeps the session locked.
    pub fn abort() {
        let attempt = {
            let mut lock = Self::write();
            let attempt = lock.attempt.take();
            if let Some(attempt) = &attempt {
                attempt.abort();
            }
            lock.checking = false;
            lock.failed = false;
            lock.unlocked = false;
            attempt
        };
        if let Some(attempt) = attempt {
            pam::abort_session(&attempt);
        }
    }

    fn begin(config: PamConfig, password: Option<String>) -> Result<(), PamError> {
        let config = config.for_lock()?;
        let attempt = Arc::new(Session::default());
        {
            let mut lock = Self::write();
            if lock.checking {
                lock.quiet();
                return Err(PamError::Busy);
            }
            lock.checking = true;
            lock.failed = false;
            lock.unlocked = false;
            lock.attempt = Some(attempt.clone());
        }
        // Views can read both services in either order, so their write guards must never overlap.
        let result = pam::start(config, password, attempt.clone(), Some(Self::finished));
        if result.is_err() {
            let mut lock = Self::write();
            if lock
                .attempt
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &attempt))
            {
                lock.attempt = None;
                lock.checking = false;
                lock.failed = true;
            } else {
                lock.quiet();
            }
        }
        result
    }

    fn finished(attempt: &Arc<Session>, result: &PamResult) {
        let mut lock = Self::write();
        if !lock
            .attempt
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, attempt))
        {
            lock.quiet();
            return;
        }
        lock.attempt = None;
        lock.checking = false;
        lock.failed = matches!(result, PamResult::Rejected(_) | PamResult::Error(_));
        lock.unlocked = !attempt.aborted() && *result == PamResult::Success;
    }
}
