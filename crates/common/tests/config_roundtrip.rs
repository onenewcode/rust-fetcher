use common::config::{AppConfig, ImConfig, LiveConfig, ThemePreference};

#[test]
fn app_config_round_trips_through_yaml() {
    let config = AppConfig {
        live: LiveConfig {
            id: "123456".to_string(),
            cookie: "sid_guard=live".to_string(),
        },
        im: ImConfig {
            cookie: "sid_guard=im".to_string(),
            receiver_id: Some("receiver-1".to_string()),
            message_text: Some("hello".to_string()),
        },
        theme: ThemePreference::Dark,
        language: "zh-CN".to_string(),
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    let decoded: AppConfig = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(decoded, config);
}

#[test]
fn load_returns_default_when_file_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    let config = AppConfig::load(&path).unwrap();

    assert_eq!(config, AppConfig::default());
}
