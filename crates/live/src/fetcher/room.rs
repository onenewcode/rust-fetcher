use anyhow::Result;
use thiserror::Error;

const REAL_ROOM_STORE_MARKER: &str = r#"roomStore\":{\"roomInfo\":{\"room\":"#;
const MIN_ROOM_ID_LEN: usize = 10;
const CLOSED_ROOM_STATUS: u32 = 4;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoomPageError {
    #[error("Live room {live_id} is currently not broadcasting or is closed")]
    Closed { live_id: String },
    #[error("Resolved suspicious roomId={candidate}, length insufficient")]
    SuspiciousRoomId { candidate: String },
    #[error("Could not find roomId in live room page")]
    MissingRoomId,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RoomPageSnapshot {
    room_id: Option<String>,
    room_status: Option<u32>,
}

pub fn resolve_room_id(page: &str, live_id: &str) -> Result<String, RoomPageError> {
    let snapshot = RoomPageSnapshot::from_page(page);

    if snapshot.indicates_closed() {
        return Err(RoomPageError::Closed {
            live_id: live_id.to_string(),
        });
    }

    let Some(room_id) = snapshot.room_id else {
        return Err(RoomPageError::MissingRoomId);
    };

    if room_id.len() < MIN_ROOM_ID_LEN {
        return Err(RoomPageError::SuspiciousRoomId { candidate: room_id });
    }

    Ok(room_id)
}

impl RoomPageSnapshot {
    fn from_page(page: &str) -> Self {
        let window = room_page_window(page);

        Self {
            room_id: find_room_id(window),
            room_status: find_numeric_field(window, "status").and_then(|value| value.parse().ok()),
        }
    }

    fn indicates_closed(&self) -> bool {
        matches!(self.room_status, Some(CLOSED_ROOM_STATUS))
    }
}

fn find_room_id(input: &str) -> Option<String> {
    let mut fallback = None;

    for start in field_positions(input, "roomId") {
        let tail = &input[start + "roomId".len()..];
        let Some(digit_start) = tail.find(|ch: char| ch.is_ascii_digit()) else {
            continue;
        };
        let digits = tail[digit_start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if digits.is_empty() {
            continue;
        }
        if digits.len() >= MIN_ROOM_ID_LEN {
            return Some(digits);
        }
        if fallback.is_none() {
            fallback = Some(digits);
        }
    }

    fallback
}

fn room_page_window(page: &str) -> &str {
    page.rfind(REAL_ROOM_STORE_MARKER)
        .map_or(page, |start| &page[start..])
}

fn find_numeric_field(input: &str, field_name: &str) -> Option<String> {
    for start in field_positions(input, field_name) {
        let tail = &input[start + field_name.len()..];
        let Some(digit_start) = tail.find(|ch: char| ch.is_ascii_digit()) else {
            continue;
        };
        let digits = tail[digit_start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            return Some(digits);
        }
    }

    None
}

fn field_positions<'a>(input: &'a str, field_name: &'a str) -> impl Iterator<Item = usize> + 'a {
    input
        .match_indices(field_name)
        .filter_map(move |(start, _)| {
            if is_token_boundary(input, start, field_name.len()) {
                Some(start)
            } else {
                None
            }
        })
}

fn is_token_boundary(input: &str, start: usize, len: usize) -> bool {
    let prev_is_ident = input[..start]
        .chars()
        .next_back()
        .is_some_and(is_identifier_char);
    let next_is_ident = input[start + len..]
        .chars()
        .next()
        .is_some_and(is_identifier_char);

    !prev_is_ident && !next_is_ident
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::{RoomPageError, resolve_room_id};

    #[test]
    fn extracts_room_id_without_regex() {
        let page = r#"prefix roomId\\":\\"7624045002674653994\\" suffix"#;

        assert_eq!(
            resolve_room_id(page, "23058211495").unwrap(),
            "7624045002674653994"
        );
    }

    #[test]
    fn extracts_room_id_from_unescaped_json_shape() {
        let page = r#"{"roomId":"7624045002674653994","status":2}"#;

        assert_eq!(
            resolve_room_id(page, "23058211495").unwrap(),
            "7624045002674653994"
        );
    }

    #[test]
    fn ignores_room_id_str_and_uses_room_id() {
        let page = r#"roomStore\":{\"roomInfo\":{\"room\":{\"roomIdStr\":\"0\"},\"roomId\":\"7624045002674653994\"}}"#;

        assert_eq!(
            resolve_room_id(page, "23058211495").unwrap(),
            "7624045002674653994"
        );
    }

    #[test]
    fn ignores_short_placeholder_room_id() {
        let page = r#"roomId=0; data={"roomId":"7624045002674653994"}"#;

        assert_eq!(
            resolve_room_id(page, "23058211495").unwrap(),
            "7624045002674653994"
        );
    }

    #[test]
    fn reports_closed_room_when_status_is_4() {
        let page = r#"roomStore\":{\"roomInfo\":{\"room\":{\"id_str\":\"7624347094471822116\",\"status\":4,\"status_str\":\"4\",\"title\":\"JueHuoLaoLiu Glory 100 Stars Room!\",\"user_count_str\":\"0\"},\"roomId\":\"7624347094471822116\",\"web_rid\":\"848782041423\"}}"#;

        assert_eq!(
            resolve_room_id(page, "23058211495"),
            Err(RoomPageError::Closed {
                live_id: "23058211495".to_string(),
            })
        );
    }

    #[test]
    fn skips_placeholder_room_store_and_uses_real_room_store() {
        let page = concat!(
            r#"roomStore\":{\"roomInfo\":{},\"liveStatus\":\"normal\"}"#,
            r#" something "#,
            r#"roomStore\":{\"roomInfo\":{\"room\":{\"id_str\":\"7624347094471822116\",\"status\":4}},\"roomId\":\"7624347094471822116\"}"#
        );

        assert_eq!(
            resolve_room_id(page, "23058211495"),
            Err(RoomPageError::Closed {
                live_id: "23058211495".to_string(),
            })
        );
    }
}
