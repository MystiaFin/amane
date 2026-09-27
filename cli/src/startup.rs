use std::fs;
use std::process::ExitCode;

use crate::paths;

const TEMPLATE: &str = include_str!("../template/main.rs");

pub fn run() -> ExitCode {
    let folder = paths::config().join("src");

    let main = folder.join("main.rs");

    // the user's own shell is never overwritten
    if main.exists() {
        eprintln!("failed to start up: {} already exists", main.display());

        return ExitCode::FAILURE;
    }

    fs::create_dir_all(&folder).expect("failed to create config folder");

    fs::write(&main, TEMPLATE).expect("failed to write main.rs");

    println!("created {}", main.display());

    ExitCode::SUCCESS
}
