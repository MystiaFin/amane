use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::BINARY;

/*
 * every command shares the one dev build, a separate release build
 * would keep a second copy of every dependency on disk, and the
 * dependencies are already optimised in dev (see project.rs)
 */

// cargo prints its own progress and errors, so this only reports success
pub fn build(project: &Path) -> bool {
    let mut command = Command::new("cargo");

    command.arg("build").current_dir(project);

    command.status().is_ok_and(|status| status.success())
}

pub fn binary(project: &Path) -> PathBuf {
    project.join("target").join("debug").join(BINARY)
}
