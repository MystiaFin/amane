use std::env;
use std::path::PathBuf;

// the user's own dir comes first, so their files win over the system's
pub fn find() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = data_home() {
        dirs.push(home);
    }

    let system = env::var("XDG_DATA_DIRS").unwrap_or_default();

    // an unset or empty XDG_DATA_DIRS means these two, as the spec says
    let system = if system.is_empty() {
        String::from("/usr/local/share:/usr/share")
    } else {
        system
    };

    for dir in system.split(':') {
        if !dir.is_empty() {
            dirs.push(PathBuf::from(dir));
        }
    }

    dirs
}

fn data_home() -> Option<PathBuf> {
    if let Ok(dir) = env::var("XDG_DATA_HOME")
        && !dir.is_empty()
    {
        return Some(PathBuf::from(dir));
    }

    let home = env::var("HOME").ok()?;

    Some(PathBuf::from(home).join(".local/share"))
}
