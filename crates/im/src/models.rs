use crate::proto::response::Response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IMMessageContent {
    pub text: String,
    #[serde(alias = "mentionUsers", default)]
    pub mention_users: Vec<String>,
    #[serde(rename = "aweType", default = "default_awe_type")]
    pub awe_type: i32,
    #[serde(alias = "richTextInfos", default)]
    pub rich_text_infos: Vec<String>,
}

fn default_awe_type() -> i32 {
    700
}

#[derive(Debug, Clone, Default)]
pub struct IMSendConfig {
    pub cookie: String,
    pub timeout: u64,
    pub receiver_id: Option<String>,
    pub conversation_id: Option<String>,
    pub message_text: Option<String>,
}

#[derive(Debug)]
pub struct SendResult {
    pub request_url: String,
    pub http_status: u16,
    pub content_type: String,
    pub response_size: usize,
    pub response_proto: Option<Response>,
    pub response_text_preview: String,
}
