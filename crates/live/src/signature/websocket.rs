use crate::error::{LiveError, Result};
use std::collections::HashMap;

use reqwest::Url;

use crate::js_engine::JsEngine;

const WEBSOCKET_SIGNATURE_PARAMS: [&str; 13] = [
    "live_id",
    "aid",
    "version_code",
    "webcast_sdk_version",
    "room_id",
    "sub_room_id",
    "sub_channel_id",
    "did_rule",
    "user_unique_id",
    "device_platform",
    "device_type",
    "ac",
    "identity",
];

/// # Errors
///
/// Returns an error if the URL is invalid or the signature generation fails.
pub async fn generate_websocket_signature(js: &JsEngine, wss_url: &str) -> Result<String> {
    let url = Url::parse(wss_url).map_err(|e| LiveError::WebSocket(format!("Invalid URL: {e}")))?;
    let query_pairs = collect_query_pairs(&url);
    let md5_stub = format!(
        "{:x}",
        md5::compute(joined_websocket_signature_params(&query_pairs))
    );
    js.websocket_signature(&md5_stub).await
}

fn collect_query_pairs(url: &Url) -> HashMap<String, String> {
    url.query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn joined_websocket_signature_params(query_pairs: &HashMap<String, String>) -> String {
    WEBSOCKET_SIGNATURE_PARAMS
        .iter()
        .map(|key| format!("{key}={}", query_pairs.get(*key).map_or("", String::as_str)))
        .collect::<Vec<_>>()
        .join(",")
}
