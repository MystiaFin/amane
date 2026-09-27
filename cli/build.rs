use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// besides src, the library needs its manifest and the lock file it was tested with
const EXTRA_FILES: [&str; 2] = ["Cargo.toml", "Cargo.lock"];

fn main() {
    let cli = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("failed to find cli folder"));

    let library = cli.parent().expect("failed to find library folder");

    let mut files = Vec::new();

    collect(library, &library.join("src"), &mut files);

    files.extend(EXTRA_FILES.map(String::from));

    let entries: String = files.iter().map(|file| entry(library, file)).collect();

    let source = format!("pub static FILES: &[(&str, &[u8])] = &[\n{entries}];\n");

    let output = PathBuf::from(env::var("OUT_DIR").expect("failed to find build output folder"));

    fs::write(output.join("library.rs"), source).expect("failed to write library list");

    // cargo looks inside the src folder too, so any edit to the library repacks it
    for watched in ["src", "Cargo.toml", "Cargo.lock"] {
        let path = library.join(watched);

        println!("cargo:rerun-if-changed={}", path.display());
    }
}

// include_bytes copies the file into the amane binary when it is compiled
fn entry(library: &Path, file: &str) -> String {
    let path = library.join(file);

    format!("    ({file:?}, include_bytes!({path:?})),\n")
}

// records every file under the folder as a path relative to the library root
fn collect(library: &Path, folder: &Path, files: &mut Vec<String>) {
    let entries = fs::read_dir(folder).expect("failed to read library folder");

    for entry in entries {
        let path = entry.expect("failed to read library entry").path();

        if path.is_dir() {
            collect(library, &path, files);

            continue;
        }

        let relative = path
            .strip_prefix(library)
            .expect("failed to make library path relative");

        files.push(relative.display().to_string());
    }
}
