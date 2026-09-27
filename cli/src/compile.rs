use std::process::ExitCode;

use crate::cargo::{self, Profile};
use crate::project;

pub fn run() -> ExitCode {
    let project = project::prepare();

    if !cargo::build(&project, Profile::Release) {
        return ExitCode::FAILURE;
    }

    let binary = cargo::binary(&project, Profile::Release);

    println!("built {}", binary.display());

    ExitCode::SUCCESS
}
