use common::config::AppConfig;
use service::commands::StartLiveCommand;

#[test]
fn start_live_command_wraps_config() {
    let config = AppConfig::default();
    let command = StartLiveCommand::new(config.clone());

    assert_eq!(command.config, config);
}
