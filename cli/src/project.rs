use std::fs;
use std::path::{Path, PathBuf};

use crate::{library, paths};

// the binary cargo builds from the user's config
pub const BINARY: &str = "amane-shell";

// unpacks the library and writes the cargo project around the user's main.rs
pub fn prepare() -> PathBuf {
    let cache = paths::cache();

    let library_folder = cache.join("library");
    let project_folder = cache.join("project");

    library::unpack(&library_folder);

    fs::create_dir_all(&project_folder).expect("failed to create project folder");

    let main = paths::config().join("src").join("main.rs");

    let main_path = main.display().to_string();
    let library_path = library_folder.display().to_string();

    let manifest = manifest(&main_path, &library_path);

    write_if_changed(&project_folder.join("Cargo.toml"), manifest.as_bytes());

    // the library's own lock file pins the versions amane was tested with
    let lock = fs::read(library_folder.join("Cargo.lock")).expect("failed to read library lock");

    write_if_changed(&project_folder.join("Cargo.lock"), &lock);

    project_folder
}

/*
 * dev builds optimise amane and its dependencies, drawing unoptimised
 * is too slow for smooth animation; only the user's own crate stays
 * unoptimised, so a save still rebuilds quickly after the first build.
 * the empty workspace keeps cargo from joining a workspace in a parent folder
 */
fn manifest(main: &str, library: &str) -> String {
    format!(
        "[package]
name = \"{BINARY}\"
version = \"0.1.0\"
edition = \"2024\"

[[bin]]
name = \"{BINARY}\"
path = '{main}'

[dependencies]
amane = {{ path = '{library}' }}

[profile.dev.package.\"*\"]
opt-level = 3

[workspace]
"
    )
}

fn write_if_changed(path: &Path, contents: &[u8]) {
    if fs::read(path).is_ok_and(|current| current == contents) {
        return;
    }

    fs::write(path, contents).expect("failed to write project file");
}
