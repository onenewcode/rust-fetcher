use crate::error::{IMError, Result};
use crate::models::SendResult;
use crate::proto::response::Response;
use prost::Message;

pub fn decode_response(bytes: &[u8]) -> Result<Response> {
    Response::decode(bytes).map_err(IMError::Protobuf)
}

pub fn decode_into_result(
    request_url: String,
    http_status: u16,
    content_type: String,
    bytes: &[u8],
) -> SendResult {
    let response_proto_result = decode_response(bytes);
    let mut preview = String::new();
    let response_proto = match response_proto_result {
        Ok(proto) => Some(proto),
        Err(_) => {
            preview = String::from_utf8_lossy(bytes)
                .chars()
                .take(1000)
                .collect::<String>();
            None
        }
    };

    SendResult {
        request_url,
        http_status,
        content_type,
        response_size: bytes.len(),
        response_proto,
        response_text_preview: preview,
    }
}

pub fn validate_result(result: &SendResult) -> Result<()> {
    if result.http_status != 200 {
        return Err(IMError::Api(
            format!("HTTP status not OK: {}", result.http_status),
            0,
        ));
    }
    let proto = result
        .response_proto
        .as_ref()
        .ok_or_else(|| IMError::Message("Response protobuf parse failed".to_string()))?;
    if proto.message != "OK" {
        return Err(IMError::Api(proto.message.clone(), proto.status_code));
    }
    Ok(())
}
