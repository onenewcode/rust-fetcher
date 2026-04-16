use std::time::Duration;

use crate::error::{IMError, Result};
use crate::models::SendResult;
use crate::response::decode_into_result;
use common::http::build_common_headers;
use reqwest::header::{ACCEPT, CONTENT_TYPE, COOKIE, HeaderValue};
use tracing::{debug, info, warn};

pub async fn post_message(
    http_client: &reqwest::Client,
    url: String,
    cookie: &str,
    timeout_secs: u64,
    body: Vec<u8>,
) -> Result<SendResult> {
    let mut headers = build_common_headers();
    headers.insert(ACCEPT, HeaderValue::from_static("application/x-protobuf"));
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-protobuf"),
    );
    headers.insert(
        COOKIE,
        HeaderValue::from_str(cookie)
            .map_err(|e| IMError::Message(format!("Invalid cookie: {e}")))?,
    );

    debug!("Sending HTTP request: POST {}", url);

    let response = http_client
        .post(&url)
        .headers(headers)
        .body(body)
        .timeout(Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|e| IMError::Fetcher(e.into()))?;

    let status = response.status();
    info!("Received response: status={}", status);

    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|e| IMError::Fetcher(e.into()))?;
    debug!("Response body size: {} bytes", bytes.len());

    let result = decode_into_result(url, status.as_u16(), content_type, &bytes);
    if result.response_proto.is_none() {
        warn!(
            "Failed to decode Protobuf, raw response preview: {}",
            result.response_text_preview
        );
    }
    Ok(result)
}
