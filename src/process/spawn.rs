use std::thread;

use super::shell;

// starts a program and moves on without waiting for it
pub fn spawn(command: &str) {
    let mut child = shell(command).spawn().expect("failed to start command");

    // a finished child that nobody waits for stays behind as a zombie
    thread::spawn(move || child.wait());
}
