use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

    let source = paths::config().join("src");
    let main = source.join("main.rs");

    let main_path = main.display().to_string();
    let library_path = library_folder.display().to_string();

    let manifest = manifest(&main_path, &library_path);

    write_if_changed(&project_folder.join("Cargo.toml"), manifest.as_bytes());

    // the library's own lock file pins the versions amane was tested with
    let lock = fs::read(library_folder.join("Cargo.lock")).expect("failed to read library lock");

    write_if_changed(&project_folder.join("Cargo.lock"), &lock);

    rebuild_if_swapped(&source, &main, &project_folder);

    project_folder
}

/*
 * cargo only rebuilds when a source is newer than the last build, so a config
 * moved or restored with its old timestamps would keep running the previous one.
 * hashing the contents catches that, and touching main.rs makes cargo notice
 */
fn rebuild_if_swapped(source: &Path, main: &Path, project: &Path) {
    let mut hasher = DefaultHasher::new();

    hash_folder(source, source, &mut hasher);

    let hash = hasher.finish().to_string();
    let stamp = project.join("source-hash");

    if fs::read_to_string(&stamp).is_ok_and(|previous| previous == hash) {
        return;
    }

    if let Ok(file) = fs::File::options().append(true).open(main) {
        let _ = file.set_modified(SystemTime::now());
    }

    write_if_changed(&stamp, hash.as_bytes());
}

// sorted so the same files always give the same hash
fn hash_folder(root: &Path, folder: &Path, hasher: &mut DefaultHasher) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };

    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();

    paths.sort();

    for path in paths {
        path.strip_prefix(root).unwrap_or(&path).hash(hasher);

        if path.is_dir() {
            hash_folder(root, &path, hasher);
        } else if let Ok(contents) = fs::read(&path) {
            contents.hash(hasher);
        }
    }
}

/*
 * the dev build (the only one) optimises amane and its dependencies, drawing unoptimised
 * is too slow for smooth animation; only the user's own crate stays
 * unoptimised, so a save still rebuilds quickly after the first build.
 * dependencies carry no debug info and the user's crate only line tables,
 * which keeps the binary small and the link on every save short.
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

[profile.dev]
debug = \"line-tables-only\"

[profile.dev.package.\"*\"]
opt-level = 3
debug = false

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
