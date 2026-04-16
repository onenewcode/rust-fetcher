use std::time::Duration;

use crate::error::Result;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message as WsMessage;

pub struct ConnectionTiming {
    pub heartbeat_interval: Duration,
    pub first_frame_deadline: Instant,
}

pub fn should_stop_after_message(
    message: Option<&std::result::Result<WsMessage, WsError>>,
) -> bool {
    matches!(message, Some(Ok(WsMessage::Close(_))))
}

pub fn connection_timing(
    now: Instant,
    heartbeat_secs: u64,
    first_frame_timeout_secs: u64,
) -> ConnectionTiming {
    ConnectionTiming {
        heartbeat_interval: Duration::from_secs(heartbeat_secs),
        first_frame_deadline: now + Duration::from_secs(first_frame_timeout_secs),
    }
}

pub async fn connect(fetcher: &mut super::DouyinLiveRustFetcher, url: &str) -> Result<()> {
    fetcher.connect_websocket(url).await
}

#[cfg(test)]
mod tests {
    use super::{connection_timing, should_stop_after_message};
    use std::time::Duration;
    use tokio::time::Instant;
    use tokio_tungstenite::tungstenite::Message as WsMessage;

    #[test]
    fn stops_on_close_frame() {
        let message = Ok::<_, tokio_tungstenite::tungstenite::Error>(WsMessage::Close(None));
        assert!(should_stop_after_message(Some(&message)));
    }

    #[test]
    fn keeps_running_for_binary_frame() {
        let message =
            Ok::<_, tokio_tungstenite::tungstenite::Error>(WsMessage::Binary(vec![1].into()));
        assert!(!should_stop_after_message(Some(&message)));
    }

    #[test]
    fn builds_expected_connection_timing() {
        let now = Instant::now();
        let timing = connection_timing(now, 10, 30);
        assert_eq!(timing.heartbeat_interval, Duration::from_secs(10));
        assert!(timing.first_frame_deadline >= now + Duration::from_secs(30));
    }
}
