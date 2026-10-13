use std::fs;
use std::io::{self, Write};
use std::path::{self, Path, PathBuf};
use std::process::ExitCode;

use crate::{paths, project, target};

const TEMPLATE: &[(&str, &str)] = &[("main.rs", include_str!("../template/main.rs"))];

// a full bar split over several files, for `amane new --example`
const EXAMPLE: &[(&str, &str)] = &[
    ("main.rs", include_str!("../template/example/src/main.rs")),
    ("bar.rs", include_str!("../template/example/src/bar.rs")),
    (
        "bar/left.rs",
        include_str!("../template/example/src/bar/left.rs"),
    ),
];

pub fn run(folder: Option<&str>, example: bool) -> ExitCode {
    let files = if example { EXAMPLE } else { TEMPLATE };

    // a folder on the command line skips the question, so scripts never wait for input
    let shell = match folder {
        Some(folder) => path::absolute(folder).expect("failed to resolve folder"),
        None => paths::config_override().unwrap_or_else(ask),
    };

    let source = shell.join("src");

    // the user's own shell is never overwritten, so every file is checked before any is written
    for (name, _) in files {
        let path = source.join(name);

        if path.exists() {
            eprintln!("failed to create shell: {} already exists", path.display());

            return ExitCode::FAILURE;
        }
    }

    for (name, contents) in files {
        let path = source.join(name);

        let parent = path.parent().expect("failed to find shell folder");

        fs::create_dir_all(parent).expect("failed to create shell folder");

        fs::write(&path, contents).expect("failed to write shell file");

        println!("created {}", path.display());
    }

    // after writing, so a failed new never moves amane away from a working shell
    remember(&shell);

    // writes the editor's Cargo.toml before the first build
    project::prepare();

    ExitCode::SUCCESS
}

// points amane at the new shell, unless the override or saved target already does
fn remember(shell: &Path) {
    if shell == paths::config() {
        return;
    }

    // the default folder needs nothing saved
    if shell == paths::default_config() {
        target::clear();
    } else {
        target::save(shell);
    }
}

// enter, 1, or no input at all (end of input in a script) keep the default
fn ask() -> PathBuf {
    let default = paths::default_config();

    println!("where should the shell go?");
    println!("  1) {} (default)", default.display());
    println!("  2) somewhere else");

    if read_answer("> ") != "2" {
        return default;
    }

    let folder = read_answer("folder: ");

    // the terminal only expands ~ on the command line, not in an answer typed here
    let folder = match folder.strip_prefix("~/") {
        Some(rest) => paths::home().join(rest),
        None => PathBuf::from(folder),
    };

    // an empty answer falls back to the default instead of the current folder
    if folder.as_os_str().is_empty() {
        return default;
    }

    path::absolute(folder).expect("failed to resolve folder")
}

fn read_answer(prompt: &str) -> String {
    print!("{prompt}");

    io::stdout().flush().expect("failed to flush stdout");

    let mut answer = String::new();

    io::stdin()
        .read_line(&mut answer)
        .expect("failed to read answer");

    answer.trim().to_string()
}
