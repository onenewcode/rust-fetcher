use crate::error::{FetcherError, Result};
use std::path::Path;

pub fn ensure_runtime_files(
    _project_root: &Path,
    config_path: &Path,
    sign_js_path: &Path,
) -> Result<()> {
    if !sign_js_path.exists() {
        return Err(FetcherError::NotFound(format!(
            "Missing signature script: {}",
            sign_js_path.display()
        )));
    }

    if !config_path.exists() {
        return Err(FetcherError::NotFound(format!(
            "Missing configuration file: {}, please copy config.yaml.example to config.yaml first",
            config_path.display()
        )));
    }

    Ok(())
}
