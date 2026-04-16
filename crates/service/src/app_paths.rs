use crate::event::LiveEvent;
use anyhow::{Result, bail};
use common::error::FetcherError;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppPaths {
    project_root: PathBuf,
}

impl AppPaths {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn config_file(&self) -> PathBuf {
        self.project_root.join("config.yaml")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.project_root.join("logs")
    }

    pub fn exports_dir(&self) -> PathBuf {
        self.project_root.join("exports")
    }

    pub fn live_sign_js(&self) -> PathBuf {
        self.project_root.join("assets/js/sign.js")
    }

    pub fn im_sign_js(&self) -> PathBuf {
        self.project_root.join("assets/js/dy_ab.js")
    }

    pub fn ensure_live_runtime_files(&self) -> Result<()> {
        ensure_file_exists(&self.live_sign_js(), |path| {
            format!("Missing signature script: {}", path.display())
        })?;
        ensure_config_exists(&self.config_file())
    }

    pub fn ensure_im_runtime_files(&self) -> Result<()> {
        ensure_file_exists(&self.im_sign_js(), |path| {
            format!("Missing IM signature script: {}", path.display())
        })?;
        ensure_config_exists(&self.config_file())
    }
}

pub fn send_im_runtime_error(
    event_tx: &tokio::sync::mpsc::UnboundedSender<LiveEvent>,
    error: impl ToString,
) {
    let _ = event_tx.send(LiveEvent::ImBulkError(crate::event::ImBulkError {
        message: error.to_string(),
    }));
}

fn ensure_config_exists(path: &Path) -> Result<()> {
    ensure_file_exists(path, |path| {
        format!(
            "Missing configuration file: {}, please copy config.yaml.example to config.yaml first",
            path.display()
        )
    })
}

fn ensure_file_exists(path: &Path, message: impl FnOnce(&Path) -> String) -> Result<()> {
    if !path.exists() {
        bail!(FetcherError::NotFound(message(path)).to_string());
    }
    Ok(())
}
