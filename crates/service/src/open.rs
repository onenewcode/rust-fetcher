use crate::app_paths::AppPaths;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn log_dir(project_root: &Path) -> PathBuf {
    AppPaths::new(project_root).logs_dir()
}

pub fn export_dir(project_root: &Path) -> PathBuf {
    AppPaths::new(project_root).exports_dir()
}

pub fn open_path(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).status()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).status()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start"])
            .arg(path)
            .status()?;
    }

    Ok(())
}
