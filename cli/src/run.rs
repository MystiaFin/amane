use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

use crate::cargo::{self, Profile};
use crate::project;

pub fn run() -> ExitCode {
    let project = project::prepare();

    // cargo finishes instantly when nothing changed since the last compile
    if !cargo::build(&project, Profile::Release) {
        return ExitCode::FAILURE;
    }

    let binary = cargo::binary(&project, Profile::Release);

    // exec replaces this process, so it only returns when the shell could not start
    let error = Command::new(&binary).exec();

    eprintln!("failed to run {}: {error}", binary.display());

    ExitCode::FAILURE
}
