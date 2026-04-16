mod defaults;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use serde::Deserialize;

use self::defaults::{
    default_browser_language, default_browser_name, default_browser_platform,
    default_browser_version, default_device_platform, default_live_id, default_screen_height,
    default_screen_width, default_tz_name, default_ws_host,
};
use crate::constants::{DEFAULT_USER_UNIQUE_ID, DEFAULT_WSS_BASE, RECOMMENDED_LOGIN_COOKIE_KEYS};
use common::cookies::parse_cookie_string;
pub use common::logging::LoggingConfig;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub live: LiveConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub client: ClientConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiveConfig {
    #[serde(default = "default_live_id")]
    pub default_live_id: String,
    #[serde(default)]
    pub im: ImConfig,
}

impl Default for LiveConfig {
    fn default() -> Self {
        Self {
            default_live_id: default_live_id(),
            im: ImConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ImConfig {
    #[serde(default = "default_im_timeout")]
    pub timeout: u64,
    pub receiver_id: Option<String>,
    pub message_text: Option<String>,
}

fn default_im_timeout() -> u64 {
    20
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub live_cookie: String,
    #[serde(default)]
    pub im_cookie: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientConfig {
    #[serde(default = "default_screen_width")]
    pub screen_width: u32,
    #[serde(default = "default_screen_height")]
    pub screen_height: u32,
    #[serde(default = "default_browser_language")]
    pub browser_language: String,
    #[serde(default = "default_browser_platform")]
    pub browser_platform: String,
    #[serde(default = "default_browser_name")]
    pub browser_name: String,
    #[serde(default = "default_browser_version")]
    pub browser_version: String,
    #[serde(default = "default_true")]
    pub browser_online: bool,
    #[serde(default = "default_true")]
    pub cookie_enabled: bool,
    #[serde(default = "default_tz_name")]
    pub tz_name: String,
    #[serde(default = "default_device_platform")]
    pub device_platform: String,
    #[serde(default)]
    pub user_unique_id: String,
    #[serde(default = "default_ws_host")]
    pub ws_host: String,
}

fn default_true() -> bool {
    true
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            screen_width: default_screen_width(),
            screen_height: default_screen_height(),
            browser_language: default_browser_language(),
            browser_platform: default_browser_platform(),
            browser_name: default_browser_name(),
            browser_version: default_browser_version(),
            browser_online: default_true(),
            cookie_enabled: default_true(),
            tz_name: default_tz_name(),
            device_platform: default_device_platform(),
            user_unique_id: String::new(),
            ws_host: default_ws_host(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(path)?;
        serde_yaml::from_str(&raw).map_err(Into::into)
    }

    pub fn validate(&self) -> Result<()> {
        if self.live.default_live_id.trim().is_empty() {
            bail!("config.yaml missing live.default_live_id");
        }

        if self.logging.enabled {
            if self.logging.directory.trim().is_empty() {
                bail!("logging.directory must be provided when logging.enabled=true");
            }
            if self.logging.file_name.trim().is_empty() {
                bail!("logging.file_name must be provided when logging.enabled=true");
            }
        }

        Ok(())
    }

    pub fn validation_warnings(&self) -> Vec<String> {
        let cookies = self.cookie_map();
        if cookies.is_empty() {
            return Vec::new();
        }

        let missing: Vec<&str> = RECOMMENDED_LOGIN_COOKIE_KEYS
            .iter()
            .copied()
            .filter(|key| !cookies.contains_key(*key))
            .collect();

        if missing.is_empty() {
            Vec::new()
        } else {
            vec![format!(
                "Current login cookie is missing recommended fields: {}. The program will continue, but may fall back to anonymous state or handshake may fail.",
                missing.join(", ")
            )]
        }
    }

    pub fn cookie_string(&self) -> &str {
        self.auth.live_cookie.trim()
    }

    pub fn cookie_map(&self) -> BTreeMap<String, String> {
        parse_cookie_string(self.cookie_string())
    }

    pub fn user_unique_id(&self) -> String {
        let configured = self.client.user_unique_id.trim();
        if !configured.is_empty() {
            return configured.to_string();
        }

        self.cookie_map()
            .get("uid_tt")
            .map(String::as_str)
            .map(str::trim)
            .filter(|uid| !uid.is_empty())
            .map_or_else(|| DEFAULT_USER_UNIQUE_ID.to_string(), str::to_string)
    }

    pub fn websocket_base_url(&self) -> &str {
        let trimmed = self.client.ws_host.trim();
        if trimmed.is_empty() {
            DEFAULT_WSS_BASE
        } else {
            trimmed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn parses_cookie_string() {
        let config = AppConfig {
            auth: super::AuthConfig {
                live_cookie: "a=1; b=2".to_string(),
                ..Default::default()
            },
            ..AppConfig::default()
        };

        let cookies = config.cookie_map();
        assert_eq!(cookies.get("a").map(String::as_str), Some("1"));
        assert_eq!(cookies.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn falls_back_to_uid_tt_for_user_unique_id() {
        let config = AppConfig {
            auth: super::AuthConfig {
                live_cookie: "uid_tt=12345".to_string(),
                ..Default::default()
            },
            ..AppConfig::default()
        };

        assert_eq!(config.user_unique_id(), "12345");
    }

    #[test]
    fn uses_configured_websocket_host() {
        let mut config = AppConfig::default();
        config.client.ws_host = "wss://example.com/webcast/im/push/v2/".to_string();

        assert_eq!(
            config.websocket_base_url(),
            "wss://example.com/webcast/im/push/v2/"
        );
    }

    #[test]
    fn reports_cookie_warnings_without_failing_validation() {
        let config = AppConfig {
            auth: super::AuthConfig {
                live_cookie: "uid_tt=12345".to_string(),
                ..Default::default()
            },
            ..AppConfig::default()
        };

        assert!(config.validate().is_ok());
        assert_eq!(config.validation_warnings().len(), 1);
    }
}
