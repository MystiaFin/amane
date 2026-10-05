use std::fs;
use std::process::ExitCode;

use crate::paths;

/*
 * only the build output goes, the compiled shell stays so `amane run` still works,
 * and the library stays because the editor reads amane from it
 */
pub fn run() -> ExitCode {
    let project = paths::cache().join("project");

    if !project.exists() {
        println!("nothing to clean");

        return ExitCode::SUCCESS;
    }

    fs::remove_dir_all(&project).expect("failed to remove build cache");

    println!("removed {}", project.display());

    ExitCode::SUCCESS
}
