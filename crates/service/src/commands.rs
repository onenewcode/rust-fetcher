use common::config::AppConfig;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct StartLiveCommand {
    pub config: AppConfig,
}

impl StartLiveCommand {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SendImCommand {
    pub config: AppConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StartBulkImCommand {
    pub csv_path: PathBuf,
    pub config: AppConfig,
}
