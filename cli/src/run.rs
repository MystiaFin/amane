use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

use crate::paths;

// only starts what `amane compile` made, building is always the user's choice
pub fn run() -> ExitCode {
    let shell = paths::shell();

    if !shell.exists() {
        eprintln!("no compiled shell yet, run `amane compile` first");

        return ExitCode::FAILURE;
    }

    // exec replaces this process, so it only returns when the shell could not start
    let error = Command::new(&shell).exec();

    eprintln!("failed to run {}: {error}", shell.display());

    ExitCode::FAILURE
}
