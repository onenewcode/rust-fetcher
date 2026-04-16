use crate::error::{LiveError, Result};
use common::config::AppConfig;

pub use common::fs::ensure_runtime_files as common_ensure_runtime_files;

/// # Errors
///
/// Returns an error if any of the required runtime files cannot be ensured.
pub fn ensure_runtime_files(
    project_root: &std::path::Path,
    config_path: &std::path::Path,
    sign_js_path: &std::path::Path,
) -> Result<()> {
    common::fs::ensure_runtime_files(project_root, config_path, sign_js_path)
        .map_err(LiveError::Fetcher)
}

pub fn sign_js_path(project_root: &std::path::Path) -> std::path::PathBuf {
    project_root.join("assets/js/sign.js")
}

pub fn config_path(project_root: &std::path::Path) -> std::path::PathBuf {
    project_root.join("config.yaml")
}

pub async fn run_live(project_root: &std::path::Path, config: AppConfig) -> Result<()> {
    let mut fetcher = crate::fetcher::DouyinLiveRustFetcher::new(project_root, config)?;
    fetcher.run().await
}
