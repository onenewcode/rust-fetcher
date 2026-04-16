use crate::error::{LiveError, Result};
use flate2::read::GzDecoder;
use futures_util::{Sink, SinkExt};
use prost::Message;
use reqwest::Url;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::debug;

use super::message::{LiveEvent, MessageContext};
use crate::proto::douyin::{PushFrame, Response};
use crate::signature::{current_unix_millis, generate_numeric_id};
use common::constants::{
    DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH, WEBCAST_SDK_VERSION, WEBCAST_UPDATE_VERSION_CODE,
    WEBCAST_VERSION_CODE,
};

struct WebsocketQueryParts {
    cursor: String,
    internal_ext: String,
    user_unique_id: String,
}

struct DecodedResponseFrame {
    log_id: u64,
    response: Response,
}

pub(super) fn build_url(base: &str, room_id: &str, user_unique_id: String) -> Result<String> {
    let query_parts = build_query_parts(room_id, user_unique_id);
    build_websocket_url(base, room_id, &query_parts)
}

pub(super) async fn send_heartbeat<S>(stream: &mut S) -> Result<()>
where
    S: Sink<WsMessage, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let heartbeat = encode_push_frame(&PushFrame {
        payload_type: "hb".to_string(),
        ..PushFrame::default()
    })?;
    debug!("Sending heartbeat");
    stream
        .send(WsMessage::Ping(heartbeat.into()))
        .await
        .map_err(|e| LiveError::WebSocket(format!("Failed to send heartbeat: {e}")))
}

pub(super) async fn handle_binary_frame<S>(stream: &mut S, payload: &[u8]) -> Result<Vec<LiveEvent>>
where
    S: Sink<WsMessage, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let Some(frame) = decode_response_frame(payload) else {
        return Ok(Vec::new());
    };

    if frame.response.need_ack {
        send_ack(stream, frame.log_id, &frame.response.internal_ext).await?;
    }

    Ok(decode_live_events(frame.response.messages_list))
}

fn build_query_parts(room_id: &str, user_unique_id: String) -> WebsocketQueryParts {
    let now_ms = current_unix_millis();
    let first_req_ms = now_ms.saturating_sub(121);
    let wrds_v = generate_numeric_id(19);

    WebsocketQueryParts {
        cursor: build_cursor(now_ms),
        internal_ext: build_internal_ext(room_id, &user_unique_id, first_req_ms, now_ms, &wrds_v),
        user_unique_id,
    }
}

fn build_websocket_url(
    base: &str,
    room_id: &str,
    query_parts: &WebsocketQueryParts,
) -> Result<String> {
    let mut url = Url::parse(base)
        .map_err(|e| LiveError::WebSocket(format!("Invalid WebSocket address: {e}")))?;

    {
        let mut query = url.query_pairs_mut();
        query.append_pair("app_name", "douyin_web");
        query.append_pair("version_code", WEBCAST_VERSION_CODE);
        query.append_pair("webcast_sdk_version", WEBCAST_SDK_VERSION);
        query.append_pair("update_version_code", WEBCAST_UPDATE_VERSION_CODE);
        query.append_pair("compress", "gzip");
        query.append_pair("device_platform", "web");
        query.append_pair("cookie_enabled", "true");
        query.append_pair("screen_width", &DEFAULT_SCREEN_WIDTH.to_string());
        query.append_pair("screen_height", &DEFAULT_SCREEN_HEIGHT.to_string());
        query.append_pair("browser_online", "true");
        query.append_pair("tz_name", "Asia/Shanghai");
        query.append_pair("cursor", &query_parts.cursor);
        query.append_pair("internal_ext", &query_parts.internal_ext);
        query.append_pair("host", "https://live.douyin.com");
        query.append_pair("aid", "6383");
        query.append_pair("live_id", "1");
        query.append_pair("did_rule", "3");
        query.append_pair("endpoint", "live_pc");
        query.append_pair("support_wrds", "1");
        query.append_pair("user_unique_id", &query_parts.user_unique_id);
        query.append_pair("im_path", "/webcast/im/fetch/");
        query.append_pair("identity", "audience");
        query.append_pair("room_id", room_id);
        query.append_pair("heartbeatDuration", "0");
    }

    Ok(url.into())
}

fn build_cursor(now_ms: u128) -> String {
    format!("t-{now_ms}_m-1_v-1_f-1")
}

fn build_internal_ext(
    room_id: &str,
    user_unique_id: &str,
    first_req_ms: u128,
    now_ms: u128,
    wrds_v: &str,
) -> String {
    format!(
        "internal_src:user_id:{user_unique_id}|first_req_ms:{first_req_ms}|fetch_time:{now_ms}|seq:1|wss_info:0-{now_ms}-0-0|wrds_v:{wrds_v}|room_id:{room_id}"
    )
}

async fn send_ack<S>(stream: &mut S, log_id: u64, internal_ext: &str) -> Result<()>
where
    S: Sink<WsMessage, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let ack = encode_push_frame(&PushFrame {
        payload_type: "ack".to_string(),
        log_id,
        payload: internal_ext.as_bytes().to_vec(),
        ..PushFrame::default()
    })?;
    stream
        .send(WsMessage::Binary(ack.into()))
        .await
        .map_err(|e| LiveError::WebSocket(format!("Failed to send ACK: {e}")))
}

fn decode_response_frame(payload: &[u8]) -> Option<DecodedResponseFrame> {
    let frame = PushFrame::decode(payload).ok()?;
    if frame.payload_type != "msg" {
        return None;
    }

    let mut decoder = GzDecoder::new(&frame.payload[..]);
    let mut decompressed = Vec::new();
    if std::io::Read::read_to_end(&mut decoder, &mut decompressed).is_err() {
        return None;
    }

    let response = Response::decode(&decompressed[..]).ok()?;
    Some(DecodedResponseFrame {
        log_id: frame.log_id,
        response,
    })
}

fn decode_live_events(messages: Vec<crate::proto::douyin::Message>) -> Vec<LiveEvent> {
    messages
        .into_iter()
        .filter_map(|msg| {
            let ctx = MessageContext {
                method: &msg.method,
                msg_id: msg.msg_id,
                msg_type: msg.msg_type,
                offset: 0, // default offset
            };
            LiveEvent::decode(&ctx, &msg.payload).ok().flatten()
        })
        .collect()
}

fn encode_push_frame(frame: &PushFrame) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    frame.encode(&mut buf).map_err(LiveError::ProtobufEncode)?;
    Ok(buf)
}
