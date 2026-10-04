use std::fs;
use std::process::ExitCode;

use crate::paths;

const TEMPLATE: &[(&str, &str)] = &[("main.rs", include_str!("../template/main.rs"))];

// a full bar split over several files, for `amane startup --example`
const EXAMPLE: &[(&str, &str)] = &[
    ("main.rs", include_str!("../template/example/main.rs")),
    ("bar.rs", include_str!("../template/example/bar.rs")),
    ("bar/left.rs", include_str!("../template/example/bar/left.rs")),
];

pub fn run(example: bool) -> ExitCode {
    let files = if example { EXAMPLE } else { TEMPLATE };

    let folder = paths::config().join("src");

    // the user's own shell is never overwritten, so every file is checked before any is written
    for (name, _) in files {
        let path = folder.join(name);

        if path.exists() {
            eprintln!("failed to start up: {} already exists", path.display());

            return ExitCode::FAILURE;
        }
    }

    for (name, contents) in files {
        let path = folder.join(name);

        let parent = path.parent().expect("failed to find config folder");

        fs::create_dir_all(parent).expect("failed to create config folder");

        fs::write(&path, contents).expect("failed to write shell file");

        println!("created {}", path.display());
    }

    ExitCode::SUCCESS
}
