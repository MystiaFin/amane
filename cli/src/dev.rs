use std::path::Path;
use std::process::{Child, Command, ExitCode};

use crate::cargo::{self, Profile};
use crate::{paths, project, watch};

pub fn run() -> ExitCode {
    let project = project::prepare();

    let source = paths::config().join("src");

    let mut shell = None;

    loop {
        // taken before building, so a save during the build still starts another one
        let before = watch::snapshot(&source);

        // a failed build keeps the old shell on screen while the error is fixed
        if cargo::build(&project, Profile::Debug) {
            stop(shell.take());

            shell = start(&project);
        }

        watch::wait_for_change(&source, before);

        println!("change found, rebuilding");
    }
}

fn start(project: &Path) -> Option<Child> {
    let binary = cargo::binary(project, Profile::Debug);

    let Ok(child) = Command::new(&binary).spawn() else {
        eprintln!("failed to start {}", binary.display());

        return None;
    };

    Some(child)
}

fn stop(shell: Option<Child>) {
    let Some(mut shell) = shell else {
        return;
    };

    let _ = shell.kill();

    // waiting collects the exit status so the old shell does not linger as a zombie
    let _ = shell.wait();
}
