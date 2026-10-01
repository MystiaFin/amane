use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use super::{data_dirs, walk};

/*
 * walking the themes takes seconds, so it happens once per run; an icon
 * installed or a theme switched while the shell runs shows after a restart
 */
static INDEX: OnceLock<HashMap<String, PathBuf>> = OnceLock::new();

// every icon name the themes have, with the best file for it
pub fn index() -> &'static HashMap<String, PathBuf> {
    INDEX.get_or_init(walk_themes)
}

fn walk_themes() -> HashMap<String, PathBuf> {
    let mut icons = HashMap::new();

    let mut roots = Vec::new();

    for dir in data_dirs::find() {
        roots.push(dir.join("icons"));
    }

    // the user's theme first, hicolor is where programs put icons of their own
    let mut themes = match user_theme() {
        Some(theme) => inherited(&theme, &roots),
        None => Vec::new(),
    };

    if !themes.iter().any(|theme| theme == "hicolor") {
        themes.push(String::from("hicolor"));
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

/*
 * the theme and every theme it inherits from, nearest first; a theme lists
 * them in its index.theme, like "Inherits=Papirus-Dark,hicolor"
 */
fn inherited(theme: &str, roots: &[PathBuf]) -> Vec<String> {
    let mut themes = vec![String::from(theme)];

    let mut next = 0;

    while next < themes.len() {
        let current = themes[next].clone();

        next += 1;

        for parent in parents(&current, roots) {
            if !themes.contains(&parent) {
                themes.push(parent);
            }
        }
    }

    themes
}

// the themes one theme names in its Inherits line, from the first folder that has it
fn parents(theme: &str, roots: &[PathBuf]) -> Vec<String> {
    for root in roots {
        let Ok(index) = fs::read_to_string(root.join(theme).join("index.theme")) else {
            continue;
        };

        for line in index.lines() {
            let Some(list) = line.strip_prefix("Inherits=") else {
                continue;
            };

            return list
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(String::from)
                .collect();
        }

        return Vec::new();
    }

    Vec::new()
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
 * an svg stays sharp at any size, so it beats every png; among svgs the
 * scalable or bigger drawing wins, since the small ones are simplified
 */
fn score(path: &Path) -> u32 {
    let is_svg = path.extension().is_some_and(|extension| extension == "svg");

    if !is_svg {
        return 1 + size(path);
    }

    let size = match size(path) {
        0 => 512,
        size => size,
    };

    10_000 + size
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

    let settings = fs::read_to_string(config.join("gtk-3.0/settings.ini")).unwrap_or_default();

    for line in settings.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        if key.trim() == "gtk-icon-theme-name" {
            return Some(value.trim().to_string());
        }
    }

    dconf_theme()
}

// where gnome and gtk 4 keep it when gtk 3's file doesn't say, like "'Papirus-Dark'"
fn dconf_theme() -> Option<String> {
    let output = Command::new("dconf")
        .args(["read", "/org/gnome/desktop/interface/icon-theme"])
        .stderr(Stdio::null())
        .output()
        .ok()?;

    let text = String::from_utf8(output.stdout).ok()?;

    let theme = text.trim().trim_matches('\'');

    if theme.is_empty() {
        return None;
    }

    Some(String::from(theme))
}
