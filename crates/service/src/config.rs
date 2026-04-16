pub use common::config::{AppConfig, ImConfig, LiveConfig, ThemePreference};

use anyhow::{Result, bail};
use std::path::Path;

pub fn load_config(path: &Path) -> Result<AppConfig> {
    Ok(AppConfig::load(path)?)
}

pub fn save_config(path: &Path, config: &AppConfig) -> Result<()> {
    let yaml = serde_yaml::to_string(config)?;
    std::fs::write(path, yaml)?;
    Ok(())
}

pub fn validate_live_config(config: &AppConfig) -> Result<()> {
    if config.live.id.trim().is_empty() {
        bail!("live.id is required");
    }
    if config.live.cookie.trim().is_empty() {
        bail!("live.cookie is required");
    }
    Ok(())
}
