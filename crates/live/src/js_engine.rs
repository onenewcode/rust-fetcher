use std::path::Path;

use crate::error::{LiveError, Result};
use common::js::{JsSigner, WebsocketSignatureRequest};

pub struct JsEngine {
    signer: JsSigner,
    script: String,
}

impl JsEngine {
    pub fn new(repo_root: &Path) -> Result<Self> {
        let script_path = repo_root.join("assets/js/sign.js");
        Ok(Self {
            signer: JsSigner::start().map_err(LiveError::Fetcher)?,
            script: std::fs::read_to_string(script_path).map_err(|error| {
                LiveError::Fetcher(common::error::FetcherError::Js(format!(
                    "Failed to read live sign script: {error}"
                )))
            })?,
        })
    }

    pub async fn websocket_signature(&self, md5_stub: &str) -> Result<String> {
        let signature = self
            .signer
            .sign_websocket(WebsocketSignatureRequest {
                script: self.script.clone(),
                md5_stub: md5_stub.to_string(),
            })
            .await
            .map_err(|e| {
                LiveError::Signature(format!("Failed to call JS function get_sign: {e}"))
            })?;
        Ok(signature)
    }
}
