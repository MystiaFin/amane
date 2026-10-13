use std::fs;
use std::path::{self, Path};
use std::process::ExitCode;

use crate::paths;

pub fn run(folder: Option<&str>) -> ExitCode {
    let Some(folder) = folder else {
        println!("{}", paths::config().display());

        return ExitCode::SUCCESS;
    };

    // a relative path would point somewhere else once amane runs from another folder
    let folder = path::absolute(folder).expect("failed to resolve folder");

    // catches typos, and new shells come from `amane new <folder>` instead
    if !folder.join("src").join("main.rs").exists() {
        eprintln!("failed to target {}: no src/main.rs there", folder.display());
        eprintln!("to create a shell in it, run `amane new {}`", folder.display());

        return ExitCode::FAILURE;
    }

    save(&folder);

    ExitCode::SUCCESS
}

pub fn save(folder: &Path) {
    let file = paths::target_file();

    let parent = file.parent().expect("failed to find state folder");

    fs::create_dir_all(parent).expect("failed to create state folder");

    fs::write(&file, folder.display().to_string()).expect("failed to save target");

    println!("amane now builds {}", folder.display());
}

// back to the default folder, which needs nothing saved
pub fn clear() {
    let _ = fs::remove_file(paths::target_file());
}
