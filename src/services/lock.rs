mod pam;

use std::env;
use std::thread;

use crate::Service;

// the pam service file in /etc/pam.d that checks the password
const PAM_SERVICE: &str = "login";

// what the lock screen shows while App::lock holds the session
#[derive(Default)]
pub struct Lock {
    checking: bool,

    // the last password was wrong
    failed: bool,

    // the backend ends the lock once this is set, only pam sets it
    unlocked: bool,
}

// only changes when a password is tried
impl Service for Lock {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl Lock {
    // pam is waiting on the password
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
     * pam can take seconds to answer, so it runs on its own thread
     * and the screen unlocks only if it accepts the password
     */
    pub fn unlock(password: &str) {
        let password = String::from(password);

        let mut lock = Self::write();

        // one try at a time
        if lock.checking {
            return;
        }

        lock.checking = true;
        lock.failed = false;

        drop(lock);

        thread::spawn(move || {
            let user = env::var("USER").expect("failed to read USER");

            let accepted = pam::authenticate(PAM_SERVICE, &user, &password);

            let mut lock = Self::write();

            lock.checking = false;
            lock.failed = !accepted;
            lock.unlocked = accepted;
        });
    }
}
