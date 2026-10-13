use std::env;
use std::fs;
use std::path::PathBuf;

// where the user writes their shell
pub fn config() -> PathBuf {
    if let Some(folder) = config_override() {
        return folder;
    }

    // set by `amane target`, for a shell kept outside ~/.config
    if let Ok(saved) = fs::read_to_string(target_file()) {
        return PathBuf::from(saved.trim_end());
    }

    default_config()
}

// lets the maintainer build another shell folder without changing the saved target
pub fn config_override() -> Option<PathBuf> {
    env::var_os("AMANE_CONFIG").map(PathBuf::from)
}

pub fn default_config() -> PathBuf {
    base("XDG_CONFIG_HOME", ".config").join("amane")
}

// where `amane target` saves the folder it was pointed at, kept per machine
pub fn target_file() -> PathBuf {
    base("XDG_STATE_HOME", ".local/state")
        .join("amane")
        .join("config-path")
}

// where amane keeps the library, the generated project and build output
pub fn cache() -> PathBuf {
    base("XDG_CACHE_HOME", ".cache").join("amane")
}

// the shell `amane compile` made, kept outside target so a clean doesn't remove it
pub fn shell() -> PathBuf {
    cache().join("amane-shell")
}

// the xdg variable wins when set, otherwise the usual folder inside home
fn base(variable: &str, fallback: &str) -> PathBuf {
    if let Some(folder) = env::var_os(variable) {
        return PathBuf::from(folder);
    }

    home().join(fallback)
}

pub fn home() -> PathBuf {
    let home = env::var_os("HOME").expect("failed to find home: HOME is not set");

    PathBuf::from(home)
}
