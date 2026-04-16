mod bootstrap;
mod connection;
mod message;
mod room;
mod session;
mod websocket;

use crate::error::{LiveError, Result};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{Instant, interval, sleep_until};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, info, warn};

use self::connection::{connection_timing, should_stop_after_message};
use self::room::resolve_room_id;
use self::session::FetchSession;
use self::websocket::{build_url as build_websocket_url, handle_binary_frame, send_heartbeat};
use crate::export::CommentUserCsvExporter;
use crate::js_engine::JsEngine;
use crate::signature::{
    current_unix_seconds, generate_ms_token, generate_websocket_signature, get_ac_signature,
    host_without_scheme,
};
use common::config::AppConfig;
use common::constants::{
    DEFAULT_USER_AGENT as UA, DOUYIN_HOST as HOST, FIRST_LIVE_FRAME_TIMEOUT_SECS,
    HEARTBEAT_INTERVAL_SECS, LIVE_URL,
};

pub struct DouyinLiveRustFetcher {
    config: AppConfig,
    session: FetchSession,
    js: JsEngine,
    comment_exporter: CommentUserCsvExporter,
    room_id: Option<String>,
}

impl DouyinLiveRustFetcher {
    /// # Errors
    ///
    /// Returns an error if the session or exporter cannot be initialized.
    pub fn new(repo_root: &std::path::Path, config: AppConfig) -> Result<Self> {
        let cookies = common::cookies::parse_cookie_string(&config.live.cookie);
        let session = FetchSession::new(cookies).map_err(LiveError::Fetcher)?;
        let comment_exporter = CommentUserCsvExporter::new(repo_root, &config.live.id);

        Ok(Self {
            config,
            session,
            js: JsEngine::new(repo_root)?,
            comment_exporter,
            room_id: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let room_id = bootstrap::ensure_room_id(self).await?;
        info!(%room_id, "Starting to connect to live WebSocket");
        debug!(
            path = %self.comment_exporter.path().display(),
            "Comment user CSV will be written as needed"
        );

        let ws_base = self.build_wss_url(&room_id)?;
        let signature = generate_websocket_signature(&self.js, &ws_base)
            .await
            .map_err(|e| LiveError::Signature(format!("{e}")))?;
        let signed_url = format!("{ws_base}&signature={signature}");
        connection::connect(self, &signed_url).await
    }

    async fn resolve_room_id_from_live_page(&self, nonce: &str) -> Result<String> {
        let site = host_without_scheme().map_err(|e| LiveError::Signature(format!("{e}")))?;
        let ac_signature = get_ac_signature(site, nonce, UA, current_unix_seconds());
        let url = self.live_page_url();
        let headers = self
            .session
            .request_headers(
                &url,
                &[
                    (
                        "msToken",
                        generate_ms_token(common::constants::MS_TOKEN_LENGTH),
                    ),
                    ("__ac_nonce", nonce.to_string()),
                    ("__ac_signature", ac_signature),
                ],
                Some(&url),
            )
            .map_err(LiveError::Fetcher)?;
        let text = self
            .session
            .get_text(
                &url,
                headers,
                "Failed to request live room page",
                "Live room page returned non-2xx status",
            )
            .await
            .map_err(LiveError::Fetcher)?;
        let room_id = resolve_room_id(&text, &self.config.live.id)
            .map_err(|e| LiveError::Room(format!("{e}")))?;
        debug!(%room_id, "Resolved room_id");
        Ok(room_id)
    }

    async fn connect_websocket(&mut self, url: &str) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);

        let request = self
            .session
            .websocket_request(url, &self.live_page_url())
            .map_err(LiveError::Fetcher)?;
        let (mut ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| LiveError::WebSocket(format!("Handshake failed: {e}")))?;

        info!("WebSocket connected successfully");

        let timing = connection_timing(
            Instant::now(),
            HEARTBEAT_INTERVAL_SECS,
            FIRST_LIVE_FRAME_TIMEOUT_SECS,
        );
        let mut heartbeat_timer = interval(timing.heartbeat_interval);
        let first_frame_deadline = timing.first_frame_deadline;

        loop {
            tokio::select! {
                _ = heartbeat_timer.tick() => {
                    send_heartbeat(&mut ws_stream).await.map_err(|e| LiveError::WebSocket(format!("{e}")))?;
                }
                msg = ws_stream.next() => {
                    if should_stop_after_message(msg.as_ref()) {
                        warn!("WebSocket connection closed");
                        break;
                    }

                    match msg {
                        Some(Ok(WsMessage::Binary(bin))) => {
                            let events = handle_binary_frame(&mut ws_stream, &bin).await.map_err(|e| LiveError::WebSocket(format!("{e}")))?;
                            for event in events {
                                if let Err(e) = tx.send(event).await {
                                    warn!("Failed to send event to processing channel: {e}");
                                }
                            }
                        }
                        Some(Err(e)) => return Err(LiveError::WebSocket(format!("Stream error: {e}"))),
                        _ => {}
                    }
                }
                Some(event) = rx.recv() => {
                    match event {
                        crate::fetcher::message::LiveEvent::Chat { record } => {
                            info!(
                                user = %record.user_name,
                                content = %record.comment_content,
                                "💬"
                            );
                            self.comment_exporter.append_comment(&record)?;
                        }
                        crate::fetcher::message::LiveEvent::StreamEnded => {
                            info!("Live stream ended by host");
                            break;
                        }
                    }
                }
                () = sleep_until(first_frame_deadline) => {
                }
            }
        }
        Ok(())
    }

    fn live_page_url(&self) -> String {
        format!("{}{}", LIVE_URL, self.config.live.id)
    }

    async fn ensure_ttwid(&self) -> Result<String> {
        if let Some(ttwid) = self
            .session
            .cookie_value(LIVE_URL, "ttwid")
            .map_err(LiveError::Fetcher)?
        {
            return Ok(ttwid);
        }

        let headers = self
            .session
            .request_headers(LIVE_URL, &[], None)
            .map_err(LiveError::Fetcher)?;
        self.session
            .warm_cookies(
                LIVE_URL,
                headers,
                "Failed to get ttwid",
                "Getting ttwid returned non-2xx status",
            )
            .await
            .map_err(LiveError::Fetcher)?;

        let ttwid = self
            .session
            .cookie_value(LIVE_URL, "ttwid")
            .map_err(LiveError::Fetcher)?
            .ok_or_else(|| {
                LiveError::Fetcher(common::error::FetcherError::Internal(
                    "Still no ttwid after request completion".to_string(),
                ))
            })?;
        debug!(%ttwid, "Successfully obtained ttwid");
        Ok(ttwid)
    }

    async fn fetch_ac_nonce(&self) -> Result<String> {
        let headers = self
            .session
            .request_headers(HOST, &[], None)
            .map_err(LiveError::Fetcher)?;
        self.session
            .warm_cookies(
                HOST,
                headers,
                "Failed to get __ac_nonce",
                "Getting __ac_nonce returned non-2xx status",
            )
            .await
            .map_err(LiveError::Fetcher)?;

        let nonce = self
            .session
            .cookie_value(HOST, "__ac_nonce")
            .map_err(LiveError::Fetcher)?
            .ok_or_else(|| {
                LiveError::Fetcher(common::error::FetcherError::Internal(
                    "Still no __ac_nonce after request completion".to_string(),
                ))
            })?;
        debug!(%nonce, "Successfully obtained __ac_nonce");
        Ok(nonce)
    }

    fn build_wss_url(&self, room_id: &str) -> Result<String> {
        build_websocket_url(
            common::constants::DEFAULT_WSS_HOST,
            room_id,
            self.config.user_unique_id(),
        )
    }
}
