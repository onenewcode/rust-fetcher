use crate::cookies::parse_cookie_string;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    #[default]
    Light,
    Dark,
    Blue,
}

impl ThemePreference {
    pub fn next(&self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Blue,
            Self::Blue => Self::Light,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Blue => "blue",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub live: LiveConfig,
    #[serde(default)]
    pub im: ImConfig,
    #[serde(default)]
    pub theme: ThemePreference,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "zh-CN".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct LiveConfig {
    pub id: String,
    pub cookie: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct ImConfig {
    pub cookie: String,
    pub receiver_id: Option<String>,
    pub message_text: Option<String>,
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&raw).map_err(Into::into)
    }

    pub fn user_unique_id(&self) -> String {
        parse_cookie_string(&self.live.cookie)
            .get("uid_tt")
            .cloned()
            .unwrap_or_else(|| "7319483754668557238".to_string())
    }
}
