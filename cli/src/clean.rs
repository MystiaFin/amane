use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

use crate::paths;

/*
 * only the build output goes, the compiled shell stays so `amane run` still works,
 * and the library stays because the editor reads amane from it
 */
pub fn run(skip_confirm: bool) -> ExitCode {
    let project = paths::cache().join("project");

    if !project.exists() {
        println!("nothing to clean");

        return ExitCode::SUCCESS;
    }

    if !skip_confirm && !confirm() {
        println!("cancelled");

        return ExitCode::SUCCESS;
    }

    fs::remove_dir_all(&project).expect("failed to remove build cache");

    println!("removed {}", project.display());

    ExitCode::SUCCESS
}

// enter, n, or no input at all (end of input in a script) all count as no
fn confirm() -> bool {
    print!(
        "the next amane compile or amane dev builds from scratch and takes a few minutes. continue? [y/N] "
    );

    io::stdout().flush().expect("failed to flush stdout");

    let mut answer = String::new();

    io::stdin()
        .read_line(&mut answer)
        .expect("failed to read answer");

    answer.trim().eq_ignore_ascii_case("y")
}
