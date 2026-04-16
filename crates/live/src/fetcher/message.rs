use prost::Message;
use thiserror::Error;
use tracing::debug;

use crate::constants::HOST;
use crate::export::CommentUserRecord;
use crate::proto::douyin::{ChatMessage, ControlMessage, User};

#[derive(Debug, Clone)]
pub struct MessageContext<'a> {
    pub method: &'a str,

    pub msg_id: i64,

    pub msg_type: i32,

    pub offset: i64,
}

#[derive(Debug, Error)]
pub enum MessageHandlingError {
    #[error("Failed to decode message {method}: {source}")]
    Decode {
        method: String,
        #[source]
        source: prost::DecodeError,
    },
}

pub enum LiveEvent {
    Chat { record: CommentUserRecord },
    StreamEnded,
}

impl LiveEvent {
    pub fn decode(
        context: &MessageContext<'_>,
        payload: &[u8],
    ) -> Result<Option<Self>, MessageHandlingError> {
        match context.method {
            "WebcastChatMessage" => Self::decode_chat(context, payload),
            "WebcastControlMessage" => Self::decode_control(context, payload),
            _ => {
                debug!(
                    method = context.method,
                    msg_id = context.msg_id,
                    msg_type = context.msg_type,
                    offset = context.offset,
                    "Ignoring unhandled message type"
                );
                Ok(None)
            }
        }
    }

    fn decode_chat(
        context: &MessageContext<'_>,
        payload: &[u8],
    ) -> Result<Option<Self>, MessageHandlingError> {
        let message = decode_protobuf::<ChatMessage>(context, payload)?;
        Ok(Some(Self::Chat {
            record: comment_user_record(&message),
        }))
    }

    fn decode_control(
        context: &MessageContext<'_>,
        payload: &[u8],
    ) -> Result<Option<Self>, MessageHandlingError> {
        let message = decode_protobuf::<ControlMessage>(context, payload)?;
        if message.status == 3 {
            Ok(Some(Self::StreamEnded))
        } else {
            Ok(None)
        }
    }
}

fn decode_protobuf<T>(
    context: &MessageContext<'_>,
    payload: &[u8],
) -> Result<T, MessageHandlingError>
where
    T: Message + Default,
{
    T::decode(payload).map_err(|source| MessageHandlingError::Decode {
        method: context.method.to_string(),
        source,
    })
}

fn comment_user_record(message: &ChatMessage) -> CommentUserRecord {
    let user = message.user.as_ref();
    CommentUserRecord {
        comment_time: comment_time(message),
        comment_content: message.content.clone(),
        user_id: comment_user_id(user),
        user_name: user_name(user),
        profile_url: profile_url(user),
    }
}

fn user_name(user: Option<&User>) -> String {
    user.map(|value| value.nick_name.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn comment_time(message: &ChatMessage) -> String {
    let common_create_time = message
        .common
        .as_ref()
        .map_or(0, |common| common.create_time);
    if common_create_time != 0 {
        return common_create_time.to_string();
    }

    if message.event_time != 0 {
        return message.event_time.to_string();
    }

    String::new()
}

fn comment_user_id(user: Option<&User>) -> String {
    let Some(user) = user else {
        return "unknown".to_string();
    };

    if !user.id_str.is_empty() {
        return user.id_str.clone();
    }
    if user.id != 0 {
        return user.id.to_string();
    }
    if !user.sec_uid.is_empty() {
        return user.sec_uid.clone();
    }

    "unknown".to_string()
}

fn profile_url(user: Option<&User>) -> String {
    let Some(sec_uid) = user
        .map(|value| value.sec_uid.trim())
        .filter(|value| !value.is_empty())
    else {
        return String::new();
    };

    format!("{HOST}user/{sec_uid}")
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::{LiveEvent, MessageContext};
    use crate::proto::douyin::{ChatMessage, Common, User};

    #[test]
    fn chat_message_captures_exportable_user_fields() {
        let message = ChatMessage {
            common: Some(Common {
                create_time: 1_700_000_001,
                ..Common::default()
            }),
            content: "hello".to_string(),
            user: Some(User {
                id: 123,
                nick_name: "alice".to_string(),
                sec_uid: "sec_uid_1".to_string(),
                id_str: "id_str_1".to_string(),
                ..User::default()
            }),
            ..ChatMessage::default()
        };
        let mut payload = Vec::new();
        message.encode(&mut payload).unwrap();

        let event = LiveEvent::decode(
            &MessageContext {
                method: "WebcastChatMessage",
                msg_id: 1,
                msg_type: 1,
                offset: 1,
            },
            &payload,
        )
        .unwrap();

        let Some(LiveEvent::Chat { record }) = event else {
            panic!("expected chat event");
        };
        assert_eq!(record.comment_time, "1700000001");
        assert_eq!(record.comment_content, "hello");
        assert_eq!(record.user_id, "id_str_1");
        assert_eq!(record.user_name, "alice");
        assert_eq!(record.profile_url, "https://www.douyin.com/user/sec_uid_1");
    }
}
