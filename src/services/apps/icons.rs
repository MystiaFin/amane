use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::{data_dirs, walk};

// every icon name the themes have, with the best file for it
pub fn index() -> HashMap<String, PathBuf> {
    let mut icons = HashMap::new();

    // the user's theme first, hicolor is where programs put icons of their own
    let mut themes = Vec::new();

    if let Some(theme) = user_theme() {
        themes.push(theme);
    }

    themes.push(String::from("hicolor"));

    let mut roots = Vec::new();

    for dir in data_dirs::find() {
        roots.push(dir.join("icons"));
    }

    for theme in &themes {
        for root in &roots {
            add_theme(&root.join(theme), &mut icons);
        }
    }

    // loose icons that belong to no theme
    for dir in data_dirs::find() {
        add_theme(&dir.join("pixmaps"), &mut icons);
    }

    icons
}

// a name found in an earlier theme stays, later themes only fill in the gaps
fn add_theme(dir: &Path, icons: &mut HashMap<String, PathBuf>) {
    let mut found: HashMap<String, PathBuf> = HashMap::new();

    for path in walk::files(dir) {
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        if !is_image(&path) {
            continue;
        }

        let better = match found.get(name) {
            Some(current) => score(&path) > score(current),
            None => true,
        };

        if better {
            found.insert(name.to_string(), path);
        }
    }

    for (name, path) in found {
        icons.entry(name).or_insert(path);
    }
}

fn is_image(path: &Path) -> bool {
    let extension = path.extension().and_then(|extension| extension.to_str());

    matches!(extension, Some("png" | "svg"))
}

/*
 * amane's images load png but not svg, so any png beats an svg,
 * and a bigger png beats a smaller one
 */
fn score(path: &Path) -> u32 {
    let is_png = path.extension().is_some_and(|extension| extension == "png");

    if !is_png {
        return 0;
    }

    1 + size(path)
}

// the size is the folder name in themes, like icons/hicolor/48x48/apps
fn size(path: &Path) -> u32 {
    for part in path.components() {
        let Some(part) = part.as_os_str().to_str() else {
            continue;
        };

        let Some((width, _)) = part.split_once('x') else {
            continue;
        };

        if let Ok(width) = width.parse() {
            return width;
        }
    }

    0
}

// the theme gtk is set to, which most desktops share
fn user_theme() -> Option<String> {
    let config = match env::var("XDG_CONFIG_HOME") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(env::var("HOME").ok()?).join(".config"),
    };

    let settings = fs::read_to_string(config.join("gtk-3.0/settings.ini")).ok()?;

    for line in settings.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        if key.trim() == "gtk-icon-theme-name" {
            return Some(value.trim().to_string());
        }
    }

    None
}
