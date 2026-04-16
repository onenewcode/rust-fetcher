use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{IMError, Result};
use crate::models::{IMMessageContent, IMSendConfig};
use crate::proto::request::{ExtValue, Request, request_body};
use prost::Message;
use uuid::Uuid;

pub fn apply_receiver_id(conversation_id: &str, receiver_id: &str) -> Result<String> {
    let mut parts: Vec<&str> = conversation_id.split(':').collect();
    if parts.len() < 4 {
        return Err(IMError::Message(format!(
            "Unsupported conversation_id format: {conversation_id}"
        )));
    }
    parts[2] = receiver_id;
    Ok(parts.join(":"))
}

pub fn apply_message_text(content: &str, message_text: &str) -> Result<String> {
    let mut payload: IMMessageContent = serde_json::from_str(content).unwrap_or_default();
    payload.text = message_text.to_string();
    serde_json::to_string(&payload).map_err(IMError::Serialization)
}

pub fn build_request_body(msg: &mut Request, config: &IMSendConfig) -> Result<Vec<u8>> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| IMError::Message(format!("System time error: {e}")))?
        .as_millis();
    let client_message_id = Uuid::new_v4().to_string();
    let stime = format!("{}.{:04}", now_ms, rand::random::<u32>() % 10000);

    msg.sequence_id = i64::try_from(now_ms).unwrap_or(i64::MAX);

    if let Some(body) = &mut msg.body
        && let Some(request_body::Body::SendMessageBody(send_body)) = &mut body.body
    {
        if let Some(receiver_id) = &config.receiver_id {
            send_body.conversation_id = apply_receiver_id(&send_body.conversation_id, receiver_id)?;
        }
        if let Some(conversation_id) = &config.conversation_id {
            send_body.conversation_id.clone_from(conversation_id);
        }
        if let Some(text) = &config.message_text {
            send_body.content = apply_message_text(&send_body.content, text)?;
        }
        send_body.client_message_id.clone_from(&client_message_id);

        let mut has_client_id_ext = false;
        let mut has_stime_ext = false;
        for ext in &mut send_body.ext {
            if ext.key == "s:client_message_id" {
                ext.value.clone_from(&client_message_id);
                has_client_id_ext = true;
            } else if ext.key == "s:stime" {
                ext.value.clone_from(&stime);
                has_stime_ext = true;
            }
        }
        if !has_client_id_ext {
            send_body.ext.push(ExtValue {
                key: "s:client_message_id".to_string(),
                value: client_message_id,
            });
        }
        if !has_stime_ext {
            send_body.ext.push(ExtValue {
                key: "s:stime".to_string(),
                value: stime,
            });
        }
    }

    let mut buf = Vec::new();
    msg.encode(&mut buf)
        .map_err(|e| IMError::Message(format!("Protobuf encode error: {e}")))?;
    Ok(buf)
}
