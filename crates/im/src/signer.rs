use std::path::PathBuf;

use crate::error::{IMError, Result};
use base64::Engine;
use common::js::{AbogusRequest, JsSigner};

pub struct ABogusSigner {
    signer: JsSigner,
    script: String,
}

impl ABogusSigner {
    pub fn new(sign_js_path: PathBuf) -> Result<Self> {
        Ok(Self {
            signer: JsSigner::start().map_err(IMError::Fetcher)?,
            script: std::fs::read_to_string(sign_js_path).map_err(|error| {
                IMError::Message(format!("Failed to read IM sign script: {error}"))
            })?,
        })
    }

    pub async fn sign(&self, query_without_abogus: &str, body_bytes: &[u8]) -> Result<String> {
        let body_b64 = base64::engine::general_purpose::STANDARD.encode(body_bytes);

        let abogus = self
            .signer
            .sign_abogus(AbogusRequest {
                script: self.script.clone(),
                query_without_abogus: query_without_abogus.to_string(),
                body_base64: body_b64,
            })
            .await
            .map_err(|e| IMError::Message(format!("Failed to call JS function get_ab: {e}")))?;

        Ok(abogus)
    }
}
