use common::config::{AppConfig, LiveConfig};
use service::event::LiveStatus;
use service::live_runtime::LiveRuntime;
use std::fs;
use std::path::PathBuf;

fn make_runtime_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("assets/js")).unwrap();
    fs::write(dir.path().join("assets/js/sign.js"), "console.log('ok');").unwrap();
    fs::write(
        dir.path().join("config.yaml"),
        "live:\n  id: \"\"\n  cookie: \"\"\n",
    )
    .unwrap();
    dir
}

#[test]
fn live_status_exposes_explicit_failed_variant() {
    assert_eq!(LiveStatus::Failed, LiveStatus::Failed);
}

#[tokio::test]
async fn start_transitions_to_running_for_valid_config() {
    let dir = make_runtime_root();
    let mut runtime = LiveRuntime::new(dir.path().to_path_buf());
    let config = AppConfig {
        live: LiveConfig {
            id: "123456".to_string(),
            cookie: "sid_guard=abc".to_string(),
        },
        ..AppConfig::default()
    };

    runtime.start(config).await.unwrap();

    assert_eq!(runtime.status(), LiveStatus::Running);
    runtime.stop().await.unwrap();
}

#[tokio::test]
async fn start_rejects_duplicate_start() {
    let dir = make_runtime_root();
    let mut runtime = LiveRuntime::new(dir.path().to_path_buf());
    let config = AppConfig {
        live: LiveConfig {
            id: "123456".to_string(),
            cookie: "sid_guard=abc".to_string(),
        },
        ..AppConfig::default()
    };

    runtime.start(config.clone()).await.unwrap();
    let error = runtime.start(config).await.unwrap_err();

    assert!(error.to_string().contains("already running"));
    runtime.stop().await.unwrap();
}

#[tokio::test]
async fn stop_without_active_session_returns_error() {
    let mut runtime = LiveRuntime::new(PathBuf::from("."));

    let error = runtime.stop().await.unwrap_err();

    assert!(error.to_string().contains("no active session"));
}

#[tokio::test]
async fn start_rejects_missing_runtime_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut runtime = LiveRuntime::new(dir.path().to_path_buf());
    let config = AppConfig {
        live: LiveConfig {
            id: "123456".to_string(),
            cookie: "sid_guard=abc".to_string(),
        },
        ..AppConfig::default()
    };

    let error = runtime.start(config).await.unwrap_err();
    assert!(
        error.to_string().contains("Missing signature script")
            || error.to_string().contains("Missing configuration file")
    );
}
