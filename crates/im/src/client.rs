use std::path::PathBuf;

use crate::error::{IMError, Result};
use crate::models::{IMSendConfig, SendResult};
use crate::request_builder::build_request_body;
use crate::request_template::load_template_proto;
use crate::response::validate_result;
use crate::signer::ABogusSigner;
use crate::transport::post_message;
use common::utils::{generate_ms_token, generate_verify_fp};
use tracing::info;

const SCHEME: &str = "https";
const NETLOC: &str = "imapi.douyin.com";
const PATH: &str = "/v1/message/send";

pub struct IMSender {
    signer: ABogusSigner,
    http_client: reqwest::Client,
}

impl IMSender {
    /// # Errors
    ///
    /// Returns an error if the `GenericJsSigner` cannot be created.
    pub fn new(sign_js_path: PathBuf) -> Result<Self> {
        Ok(Self {
            signer: ABogusSigner::new(sign_js_path)?,
            http_client: reqwest::Client::new(),
        })
    }

    /// Sends an IM message based on the provided configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The cookie is empty.
    /// - Protobuf template loading fails.
    /// - Dynamic request building fails.
    /// - JS signature generation fails.
    /// - HTTP request fails or returns a non-OK status.
    /// - Douyin API returns a business error.
    pub async fn send(&self, config: &IMSendConfig) -> Result<SendResult> {
        let cookie = &config.cookie;
        if cookie.is_empty() {
            return Err(IMError::Message("cookie is empty".to_string()));
        }

        info!(
            "Preparing to send message: receiver_id={:?}, message_text={:?}",
            config.receiver_id.as_deref().unwrap_or("default"),
            config.message_text.as_deref().unwrap_or("default")
        );

        let mut request_proto = load_template_proto()?;
        let body_bytes = build_request_body(&mut request_proto, config)?;

        let ms_token = generate_ms_token(107);
        let verify_fp = generate_verify_fp();

        let query_without_ab = format!("msToken={ms_token}&verifyFp={verify_fp}&fp={verify_fp}");
        let abogus = self.signer.sign(&query_without_ab, &body_bytes).await?;

        let mut encoded_abogus = String::new();
        url::form_urlencoded::byte_serialize(abogus.as_bytes())
            .for_each(|s| encoded_abogus.push_str(s));

        let final_query = format!("{query_without_ab}&a_bogus={encoded_abogus}");
        let final_url = format!("{SCHEME}://{NETLOC}{PATH}?{final_query}");

        let result = post_message(
            &self.http_client,
            final_url,
            cookie,
            config.timeout,
            body_bytes,
        )
        .await?;

        validate_result(&result)?;

        info!("✅ Message sent successfully");
        Ok(result)
    }
}
