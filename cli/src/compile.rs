use std::process::ExitCode;

use crate::cargo;
use crate::project;

pub fn run() -> ExitCode {
    let project = project::prepare();

    if !cargo::build(&project) {
        return ExitCode::FAILURE;
    }

    let binary = cargo::binary(&project);

    println!("built {}", binary.display());

    ExitCode::SUCCESS
}
