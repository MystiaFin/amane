use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use crate::{Lock, Pam, PamConfig, PamError, PamMessageKind, PamResult, Service};

static SERIAL: Mutex<()> = Mutex::new(());
static NEXT: AtomicUsize = AtomicUsize::new(0);

// Every test uses its own PAM policy and module; no system accounts or policies are changed.
struct Fixture {
    directory: PathBuf,
    _serial: MutexGuard<'static, ()>,
}

impl Fixture {
    fn new(mode: &str) -> Self {
        let serial = SERIAL.lock().unwrap_or_else(PoisonError::into_inner);
        let directory = std::env::temp_dir().join(format!(
            "amane-pam-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&directory).expect("create test policy directory");
        let source = directory.join("module.c");
        let module = directory.join("module.so");
        fs::write(&source, include_str!("module.c")).expect("write test module");
        let output = Command::new("cc")
            .args(["-shared", "-fPIC", "-Wall", "-Wextra", "-Werror"])
            .arg(&source)
            .arg("-o")
            .arg(&module)
            .arg("-lpam")
            .output()
            .expect("compile PAM test module");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Lock::reset();
        let fixture = Self {
            directory,
            _serial: serial,
        };
        fixture.policy(mode);
        fixture
    }

    fn config(&self) -> PamConfig {
        PamConfig::new("amane-test").config_directory(&self.directory)
    }

    fn policy(&self, mode: &str) {
        let module = self.directory.join("module.so");
        let release = self.directory.join("release");
        fs::write(
            self.directory.join("amane-test"),
            format!(
                "auth required {} {mode} {}\naccount required {} {mode}\n",
                module.display(),
                release.display(),
                module.display(),
            ),
        )
        .expect("write test policy");
    }

    fn release(&self) {
        fs::write(self.directory.join("release"), "").expect("release test module");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        Pam::abort();
        self.release();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Pam::read().active() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn wait_for(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "authentication did not reach expected state"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

fn prompt(text: &str) -> u64 {
    wait_for(|| {
        Pam::read()
            .prompt()
            .is_some_and(|message| message.text() == text)
    });
    Pam::read().prompt().expect("pending prompt").id()
}

fn complete() -> PamResult {
    wait_for(|| !Pam::read().active());
    Pam::read().result().expect("authentication result").clone()
}

#[test]
fn each_prompt_gets_its_own_response_and_messages_are_preserved() {
    let fixture = Fixture::new("interactive");
    Pam::start(fixture.config().user("amane-test-user")).expect("start authentication");
    let password = prompt("Password:");
    assert_eq!(
        Pam::read().prompt().unwrap().kind(),
        PamMessageKind::Prompt { visible: false }
    );
    assert_eq!(
        Pam::respond(password, "bad\0response"),
        Err(PamError::InvalidInput("response"))
    );
    Pam::respond(password, "test-password").expect("answer password");
    let otp = prompt("Code:");
    assert_eq!(
        Pam::read().prompt().unwrap().kind(),
        PamMessageKind::Prompt { visible: true }
    );
    assert_eq!(
        Pam::respond(password, "stale-password"),
        Err(PamError::NoPrompt)
    );
    Pam::respond(otp, "246810").expect("answer code");
    assert_eq!(complete(), PamResult::Success);
    let pam = Pam::read();
    let messages: Vec<_> = pam
        .messages()
        .iter()
        .map(|message| (message.text(), message.kind()))
        .collect();
    assert_eq!(
        messages,
        [
            ("Read this", PamMessageKind::Info),
            ("Example warning", PamMessageKind::Error),
            ("Password:", PamMessageKind::Prompt { visible: false }),
            ("Code:", PamMessageKind::Prompt { visible: true }),
        ]
    );
    assert!(pam.prompt().is_none());
    assert!(!Lock::read().unlocked());
}

#[test]
fn cancellation_wakes_a_waiting_prompt_and_allows_a_new_attempt() {
    let fixture = Fixture::new("interactive");
    Pam::start(fixture.config()).expect("start authentication");
    let old = prompt("Password:");
    assert_eq!(Pam::start(fixture.config()), Err(PamError::Busy));
    Pam::abort();
    assert_eq!(Pam::read().result(), Some(&PamResult::Aborted));
    assert_eq!(Pam::respond(old, "test-password"), Err(PamError::NoPrompt));
    assert_eq!(complete(), PamResult::Aborted);
    Pam::start(fixture.config()).expect("start another attempt");
    let current = prompt("Password:");
    assert_ne!(current, old);
    assert_eq!(Pam::respond(old, "test-password"), Err(PamError::NoPrompt));
    Pam::respond(current, "test-password").expect("answer first prompt");
    prompt("Code:");
    Pam::abort();
    assert_eq!(complete(), PamResult::Aborted);
}

#[test]
fn authentication_rejection_is_distinct_from_a_pam_error() {
    let fixture = Fixture::new("denied");
    Pam::start(fixture.config()).expect("start authentication");
    assert!(matches!(
        complete(),
        PamResult::Rejected(PamError::Native { code: 7, .. })
    ));
    fixture.policy("error");
    Pam::start(fixture.config()).expect("start error case");
    assert!(matches!(
        complete(),
        PamResult::Error(PamError::Native { code: 4, .. })
    ));
}

#[test]
fn account_rejection_does_not_unlock_the_session() {
    let fixture = Fixture::new("account-denied");
    Lock::authenticate(fixture.config()).expect("start lock authentication");
    assert!(matches!(
        complete(),
        PamResult::Rejected(PamError::Native { code: 13, .. })
    ));
    wait_for(|| !Lock::read().checking());
    assert!(Lock::read().failed());
    assert!(!Lock::read().unlocked());
}

#[test]
fn lock_authentication_unlocks_only_after_both_responses() {
    let fixture = Fixture::new("interactive");
    Lock::authenticate(fixture.config()).expect("start lock authentication");
    Pam::respond(prompt("Password:"), "test-password").expect("answer password");
    let otp = prompt("Code:");
    assert!(!Lock::read().unlocked());
    Pam::respond(otp, "246810").expect("answer code");
    assert_eq!(complete(), PamResult::Success);
    wait_for(|| Lock::read().unlocked());
}

#[test]
fn cancelling_a_lock_attempt_discards_late_success() {
    let fixture = Fixture::new("delayed");
    Lock::authenticate(fixture.config()).expect("start lock authentication");
    wait_for(|| {
        Pam::read()
            .messages()
            .iter()
            .any(|message| message.text() == "Finishing")
    });
    Lock::abort();
    fixture.release();
    assert_eq!(complete(), PamResult::Aborted);
    assert!(!Lock::read().checking());
    assert!(!Lock::read().unlocked());
}

#[test]
fn a_new_lock_cannot_be_unlocked_by_an_old_attempt() {
    let fixture = Fixture::new("delayed");
    Lock::authenticate(fixture.config()).expect("start lock authentication");
    wait_for(|| {
        Pam::read()
            .messages()
            .iter()
            .any(|message| message.text() == "Finishing")
    });
    Lock::reset();
    fixture.release();
    assert_eq!(complete(), PamResult::Aborted);
    assert!(!Lock::read().unlocked());
}

#[test]
fn lock_authentication_rejects_a_different_user() {
    let fixture = Fixture::new("success");
    assert_eq!(
        Lock::authenticate(fixture.config().user("different-user")),
        Err(PamError::InvalidInput("lock user"))
    );
    assert!(!Lock::read().unlocked());
}

#[test]
fn a_pam_module_cannot_change_the_authenticated_user() {
    let fixture = Fixture::new("change-user");
    Lock::authenticate(fixture.config()).expect("start lock authentication");
    assert_eq!(complete(), PamResult::Error(PamError::UserChanged));
    wait_for(|| !Lock::read().checking());
    assert!(!Lock::read().unlocked());
}

#[test]
fn invalid_configuration_does_not_start_a_worker() {
    let _serial = SERIAL.lock().unwrap_or_else(PoisonError::into_inner);
    for service in ["", "bad\0service", "../login"] {
        assert_eq!(
            Pam::start(PamConfig::new(service)),
            Err(PamError::InvalidInput("service"))
        );
    }
    assert_eq!(
        Pam::start(PamConfig::new("login").user("bad\0user")),
        Err(PamError::InvalidInput("user"))
    );
    assert_eq!(Pam::respond(0, "unused"), Err(PamError::NoPrompt));
    assert!(!Pam::read().active());
}

#[test]
fn password_convenience_answers_prompts_without_ui_input() {
    let fixture = Fixture::new("password-only");
    super::start(
        fixture.config(),
        Some(String::from("test-password")),
        Arc::new(super::Session::default()),
        None,
    )
    .expect("start password authentication");
    assert_eq!(complete(), PamResult::Success);
    assert!(Pam::read().prompt().is_none());
}

#[test]
fn configured_user_reaches_the_pam_module() {
    let fixture = Fixture::new("check-user");
    Pam::start(fixture.config().user("amane-test-user")).expect("start chosen user");
    assert_eq!(complete(), PamResult::Success);
    Pam::start(fixture.config().user("other-test-user")).expect("start another user");
    assert!(matches!(complete(), PamResult::Rejected(_)));
}

#[test]
fn unsupported_pam_message_styles_fail_authentication() {
    let fixture = Fixture::new("unknown-style");
    Pam::start(fixture.config()).expect("start authentication");
    assert!(matches!(
        complete(),
        PamResult::Error(PamError::Native { code: 19, .. })
    ));
}

#[test]
fn a_new_attempt_clears_the_previous_unlock_result() {
    let fixture = Fixture::new("success");
    Lock::authenticate(fixture.config()).expect("start lock authentication");
    assert_eq!(complete(), PamResult::Success);
    wait_for(|| Lock::read().unlocked());
    fixture.policy("delayed");
    Lock::authenticate(fixture.config()).expect("start another lock attempt");
    assert!(!Lock::read().unlocked());
    Lock::abort();
    fixture.release();
    assert_eq!(complete(), PamResult::Aborted);
}

#[test]
fn abort_clears_success_before_the_compositor_consumes_it() {
    let fixture = Fixture::new("success");
    Lock::authenticate(fixture.config()).expect("start lock authentication");
    assert_eq!(complete(), PamResult::Success);
    wait_for(|| Lock::read().unlocked());
    Lock::abort();
    assert!(!Lock::read().unlocked());
}

#[test]
fn a_view_can_read_lock_while_authentication_waits_for_pam() {
    let fixture = Fixture::new("interactive");
    let pam = Pam::read();
    let config = fixture.config();
    let authentication = thread::spawn(move || Lock::authenticate(config));
    let deadline = Instant::now() + Duration::from_secs(5);
    let readable = loop {
        if crate::services::store::find::<Lock>()
            .try_read()
            .is_ok_and(|lock| lock.checking())
        {
            break true;
        }
        if Instant::now() >= deadline {
            break false;
        }
        thread::sleep(Duration::from_millis(5));
    };
    drop(pam);
    authentication
        .join()
        .unwrap()
        .expect("start authentication");
    Lock::abort();
    assert_eq!(complete(), PamResult::Aborted);
    assert!(readable, "authentication held Lock while waiting for Pam");
}

#[test]
fn cancelling_before_pam_publication_cannot_unlock() {
    let fixture = Fixture::new("success");
    Pam::start(fixture.config()).expect("start earlier authentication");
    assert_eq!(complete(), PamResult::Success);
    let pam = Pam::read();
    let config = fixture.config();
    let authentication = thread::spawn(move || Lock::authenticate(config));
    wait_for(|| {
        crate::services::store::find::<Lock>()
            .try_read()
            .is_ok_and(|lock| lock.checking())
    });
    let cancellation = thread::spawn(Lock::abort);
    wait_for(|| {
        crate::services::store::find::<Lock>()
            .try_read()
            .is_ok_and(|lock| !lock.checking())
    });
    drop(pam);
    authentication
        .join()
        .unwrap()
        .expect("start cancelled authentication");
    cancellation.join().unwrap();
    assert_eq!(complete(), PamResult::Success);
    assert!(!Lock::read().unlocked());
}

#[test]
fn cancelled_reservation_does_not_replace_a_newer_result() {
    let fixture = Fixture::new("success");
    let old = Arc::new(super::Session::default());
    old.abort();
    Pam::start(fixture.config()).expect("start newer authentication");
    assert_eq!(complete(), PamResult::Success);
    super::start(fixture.config(), None, old, None).expect("resume cancelled reservation");
    assert_eq!(complete(), PamResult::Success);
}
