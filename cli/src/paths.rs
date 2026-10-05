use std::env;
use std::path::PathBuf;

// where the user writes their shell
pub fn config() -> PathBuf {
    // lets an example shell live anywhere without swapping the real config out
    if let Some(folder) = env::var_os("AMANE_CONFIG") {
        return PathBuf::from(folder);
    }

    base("XDG_CONFIG_HOME", ".config").join("amane")
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

    let home = env::var_os("HOME").expect("failed to find home: HOME is not set");

    PathBuf::from(home).join(fallback)
}
