use std::env;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::PamError;

#[derive(Debug, Clone)]
pub struct PamConfig {
    service: String,
    user: Option<String>,
    directory: Option<PathBuf>,
}

pub(super) struct PreparedConfig {
    pub service: CString,
    pub user: CString,
    pub directory: Option<CString>,
}

impl PamConfig {
    // the name of a policy file in /etc/pam.d, or in config_directory
    pub fn new(service: &str) -> Self {
        Self {
            service: String::from(service),
            user: None,
            directory: None,
        }
    }

    pub fn user(mut self, user: &str) -> Self {
        self.user = Some(String::from(user));
        self
    }

    pub fn config_directory(mut self, directory: impl AsRef<Path>) -> Self {
        self.directory = Some(directory.as_ref().to_path_buf());
        self
    }

    pub(in crate::services) fn for_lock(mut self) -> Result<Self, PamError> {
        let user = current_user()?;
        if self.user.as_ref().is_some_and(|chosen| *chosen != user) {
            return Err(PamError::InvalidInput("lock user"));
        }
        self.user = Some(user);
        Ok(self)
    }

    pub(super) fn prepare(self) -> Result<PreparedConfig, PamError> {
        if self.service.is_empty() || self.service.contains('/') {
            return Err(PamError::InvalidInput("service"));
        }
        let service = CString::new(self.service).map_err(|_| PamError::InvalidInput("service"))?;
        let user = self.user.map_or_else(current_user, Ok)?;
        if user.is_empty() {
            return Err(PamError::InvalidInput("user"));
        }
        let user = CString::new(user).map_err(|_| PamError::InvalidInput("user"))?;
        let directory = self
            .directory
            .map(|path| {
                CString::new(path.as_os_str().as_bytes())
                    .map_err(|_| PamError::InvalidInput("configuration directory"))
            })
            .transpose()?;
        Ok(PreparedConfig {
            service,
            user,
            directory,
        })
    }
}

pub(super) fn current_user() -> Result<String, PamError> {
    env::var("USER").map_err(|_| PamError::UserUnavailable)
}
