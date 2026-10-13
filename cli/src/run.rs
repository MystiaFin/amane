use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode, Stdio};

use amane::ipc_socket;

use crate::paths;

// only starts what `amane compile` made, building is always the user's choice
pub fn run() -> ExitCode {
    let shell = paths::shell();

    if !shell.exists() {
        eprintln!("no compiled shell yet, run `amane compile` first");

        return ExitCode::FAILURE;
    }

    // the shell's own "already running" error goes to /dev/null, so it is checked here
    if UnixStream::connect(ipc_socket()).is_ok() {
        eprintln!("amane is already running, use `amane restart` to restart it");

        return ExitCode::FAILURE;
    }

    /*
     * its own process group keeps ctrl+c and closing the terminal
     * away from the shell, and a closed terminal would make prints fail
     */
    let started = Command::new(&shell)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    if let Err(error) = started {
        eprintln!("failed to run {}: {error}", shell.display());

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
