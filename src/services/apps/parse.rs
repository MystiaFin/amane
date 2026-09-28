use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{DesktopApp, exec};

// the desktop file id: its path inside applications/, with / turned into -
pub fn id(applications: &Path, path: &Path) -> Option<String> {
    if path.extension()? != "desktop" {
        return None;
    }

    let relative = path.strip_prefix(applications).ok()?;

    let id = relative.to_str()?.replace('/', "-");

    Some(id)
}

// none for files that are not programs, or that ask not to be shown in menus
pub fn read(path: &Path, icons: &HashMap<String, PathBuf>) -> Option<DesktopApp> {
    let text = fs::read_to_string(path).ok()?;

    let fields = main_section(&text);

    if fields.get("Type").copied() != Some("Application") {
        return None;
    }

    if is_true(fields.get("NoDisplay")) || is_true(fields.get("Hidden")) {
        return None;
    }

    let name = fields.get("Name")?.to_string();
    let exec = exec::strip_field_codes(fields.get("Exec")?);

    let icon = fields.get("Icon").map(|icon| icon.to_string());

    let icon_path = icon.as_deref().and_then(|icon| find_icon(icon, icons));

    Some(DesktopApp {
        name,
        exec,
        icon,
        icon_path,
    })
}

// the keys of [Desktop Entry], leaving out translations like Name[de] and the action sections
fn main_section(text: &str) -> HashMap<&str, &str> {
    let mut fields = HashMap::new();

    let mut inside = false;

    for line in text.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            inside = line == "[Desktop Entry]";

            continue;
        }

        if !inside {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        fields.insert(key.trim(), value.trim());
    }

    fields
}

fn is_true(value: Option<&&str>) -> bool {
    value.copied() == Some("true")
}

// an icon can be a full path instead of a name from the theme
fn find_icon(icon: &str, icons: &HashMap<String, PathBuf>) -> Option<PathBuf> {
    let path = Path::new(icon);

    if path.is_absolute() {
        return Some(path.to_path_buf());
    }

    icons.get(icon).cloned()
}
