use std::os::unix::net::UnixStream;
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

use amane::ipc_socket;

use crate::{paths, run};

pub fn run() -> ExitCode {
    stop();

    // still running after the wait means run reports it, so there is no second error here
    run::run()
}

// matches the full path, so the shell `amane dev` runs under the same name is left alone
fn stop() {
    let shell = paths::shell();

    let status = Command::new("pkill")
        .arg("-xf")
        .arg(&shell)
        .status()
        .expect("failed to run pkill");

    // pkill exits with 1 when nothing matched, so there is nothing to wait for
    if !status.success() {
        return;
    }

    wait_until_stopped();
}

/*
 * pkill only sends the signal, the old shell takes a moment to exit,
 * and a new shell started before then would refuse to run
 */
fn wait_until_stopped() {
    let deadline = Instant::now() + Duration::from_secs(3);

    // the socket stops answering once the old shell has exited
    while UnixStream::connect(ipc_socket()).is_ok() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
}
