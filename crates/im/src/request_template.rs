use crate::error::{IMError, Result};
use crate::proto::request::Request;
use base64::Engine;
use prost::Message;

const REQUEST_PROTO_B64: &str = include_str!("request_template.b64");

pub fn load_template_proto() -> Result<Request> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(REQUEST_PROTO_B64.trim())
        .map_err(|e| IMError::Message(format!("Failed to decode REQUEST_PROTO_B64: {e}")))?;
    Request::decode(&*decoded).map_err(IMError::Protobuf)
}
