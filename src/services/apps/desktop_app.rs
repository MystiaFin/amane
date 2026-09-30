use std::path::{Path, PathBuf};

use crate::spawn;

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopApp {
    pub(crate) name: String,
    pub(crate) exec: String,
    pub(crate) icon: Option<String>,
    pub(crate) icon_path: Option<PathBuf>,
    pub(crate) description: Option<String>,
}

impl DesktopApp {
    pub fn name(&self) -> &str {
        &self.name
    }

    // the command line, without the %f style placeholders
    pub fn exec(&self) -> &str {
        &self.exec
    }

    // the icon's name in the icon theme, like "firefox"
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    // the image file for the icon, a png when the theme has one
    pub fn icon_path(&self) -> Option<&Path> {
        self.icon_path.as_deref()
    }

    // the entry's Comment, or its GenericName like "Web Browser" when it has none
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    // starts the program and moves on without waiting for it
    pub fn launch(&self) {
        spawn(&self.exec);
    }
}
