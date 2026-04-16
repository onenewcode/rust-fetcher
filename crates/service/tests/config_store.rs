use common::config::{AppConfig, ImConfig, LiveConfig, ThemePreference};
use service::config_store::{load_config, save_config, validate_live_config};
use tempfile::tempdir;

#[test]
fn load_config_returns_default_for_missing_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    let config = load_config(&path).unwrap();

    assert!(config.live.id.is_empty());
    assert!(config.live.cookie.is_empty());
}

#[test]
fn save_config_preserves_all_sections() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.yaml");
    let config = AppConfig {
        live: LiveConfig {
            id: "live-1".to_string(),
            cookie: "live-cookie".to_string(),
        },
        im: ImConfig {
            cookie: "im-cookie".to_string(),
            receiver_id: Some("receiver".to_string()),
            message_text: Some("hello".to_string()),
        },
        theme: ThemePreference::Blue,
        language: "zh-CN".to_string(),
    };

    save_config(&path, &config).unwrap();
    let loaded = load_config(&path).unwrap();

    assert_eq!(loaded, config);
}

#[test]
fn validate_live_config_rejects_missing_room_id() {
    let config = AppConfig {
        live: LiveConfig {
            id: String::new(),
            cookie: "sid_guard=abc".to_string(),
        },
        ..AppConfig::default()
    };

    let error = validate_live_config(&config).unwrap_err();
    assert!(error.to_string().contains("live.id"));
}

#[test]
fn validate_live_config_rejects_missing_cookie() {
    let config = AppConfig {
        live: LiveConfig {
            id: "123456".to_string(),
            cookie: String::new(),
        },
        ..AppConfig::default()
    };

    let error = validate_live_config(&config).unwrap_err();
    assert!(error.to_string().contains("live.cookie"));
}
