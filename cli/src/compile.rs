use std::fs;
use std::process::ExitCode;

use crate::cargo;
use crate::{paths, project};

pub fn run() -> ExitCode {
    let project = project::prepare();

    if !cargo::build(&project) {
        return ExitCode::FAILURE;
    }

    let binary = cargo::binary(&project);
    let shell = paths::shell();
    let fresh = shell.with_extension("new");

    /*
     * writing over a running binary fails with "text file busy",
     * so the copy goes next to it and a rename swaps it in
     */
    fs::copy(&binary, &fresh).expect("failed to copy shell");

    fs::rename(&fresh, &shell).expect("failed to replace shell");

    println!("built {}", shell.display());

    ExitCode::SUCCESS
}
