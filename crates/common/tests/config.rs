use common::config::{AppConfig, ThemePreference};

#[test]
fn app_config_defaults_theme_to_light() {
    let config = AppConfig::default();

    assert_eq!(config.theme, ThemePreference::Light);
}

#[test]
fn app_config_rejects_removed_theme_variants() {
    let result: Result<AppConfig, _> = serde_yaml::from_str(
        r#"
theme: sepia
"#,
    );

    assert!(result.is_err());
}
