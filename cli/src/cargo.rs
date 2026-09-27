use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::BINARY;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    // builds fast, runs slower, used while developing
    #[default]
    Debug,

    // builds slow, runs fast, used for the real shell
    Release,
}

// cargo prints its own progress and errors, so this only reports success
pub fn build(project: &Path, profile: Profile) -> bool {
    let mut command = Command::new("cargo");

    command.arg("build").current_dir(project);

    if profile == Profile::Release {
        command.arg("--release");
    }

    command.status().is_ok_and(|status| status.success())
}

pub fn binary(project: &Path, profile: Profile) -> PathBuf {
    let folder = match profile {
        Profile::Debug => "debug",
        Profile::Release => "release",
    };

    project.join("target").join(folder).join(BINARY)
}
