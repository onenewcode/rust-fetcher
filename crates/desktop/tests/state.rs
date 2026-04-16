use desktop::state::{AppState, reduce_event};
use service::config::{AppConfig, LiveConfig, ThemePreference};
use service::event::{
    BulkPhase, BulkProgress, ImBulkError, LiveEvent, LiveLog, LiveStatus, LogLevel,
};

#[test]
fn reduce_event_stores_live_status_directly() {
    let mut state = AppState::default();
    reduce_event(&mut state, LiveEvent::StatusChanged(LiveStatus::Running));
    assert_eq!(state.live_status, LiveStatus::Running);
}

#[test]
fn log_event_sets_error_message() {
    let mut state = AppState::default();

    reduce_event(
        &mut state,
        LiveEvent::Log(LiveLog {
            level: LogLevel::Error,
            message: "boom".to_string(),
        }),
    );

    assert_eq!(state.error_message.as_deref(), Some("boom"));
}

#[test]
fn loaded_config_populates_form_state() {
    let state = AppState::from_config(AppConfig {
        live: LiveConfig {
            id: "room-1".to_string(),
            cookie: "sid_guard=abc".to_string(),
        },
        ..AppConfig::default()
    });

    assert_eq!(state.config.live.id, "room-1");
    assert_eq!(state.config.live.cookie, "sid_guard=abc");
}

#[test]
fn loaded_config_populates_selected_theme() {
    let state = AppState::from_config(AppConfig {
        live: LiveConfig {
            id: "room-1".to_string(),
            cookie: "sid_guard=abc".to_string(),
        },
        theme: ThemePreference::Dark,
        ..AppConfig::default()
    });

    assert_eq!(state.selected_theme, ThemePreference::Dark);
}

#[test]
fn metrics_event_updates_summary_fields() {
    let mut state = AppState::default();

    reduce_event(
        &mut state,
        LiveEvent::Metrics {
            message_count: 9,
            last_event_at: Some("2026-04-13T12:00:00Z".to_string()),
        },
    );

    assert_eq!(state.message_count, 9);
    assert_eq!(state.last_event_at.as_deref(), Some("2026-04-13T12:00:00Z"));
}

#[test]
fn bulk_progress_event_updates_structured_state() {
    let mut state = AppState::default();

    reduce_event(
        &mut state,
        LiveEvent::ImBulkProgress(BulkProgress {
            progress: 1,
            total: 3,
            phase: BulkPhase::Sending {
                user_name: "alice".to_string(),
            },
        }),
    );

    assert_eq!(state.bulk_progress, 1);
    assert_eq!(state.bulk_total, 3);
    assert_eq!(
        state.bulk_status_text(),
        Some("Sending to alice (1/3)".to_string())
    );
}

#[test]
fn bulk_completed_event_formats_summary_in_desktop() {
    let mut state = AppState {
        is_bulk_sending: true,
        ..AppState::default()
    };

    reduce_event(
        &mut state,
        LiveEvent::ImBulkCompleted {
            success_count: 2,
            fail_count: 1,
        },
    );

    assert!(!state.is_bulk_sending);
    assert_eq!(
        state.bulk_status_text(),
        Some("Completed: 2 success, 1 failed".to_string())
    );
}

#[test]
fn bulk_error_event_stops_sending_and_preserves_structured_status() {
    let mut state = AppState {
        is_bulk_sending: true,
        bulk_status: Some(BulkPhase::Waiting { seconds: 7 }),
        ..AppState::default()
    };

    reduce_event(
        &mut state,
        LiveEvent::ImBulkError(ImBulkError {
            message: "boom".to_string(),
        }),
    );

    assert!(!state.is_bulk_sending);
    assert_eq!(state.error_message.as_deref(), Some("boom"));
    assert_eq!(
        state.bulk_status_text(),
        Some("Waiting 7s before next...".to_string())
    );
}

#[test]
fn app_state_defaults_to_idle_status_label() {
    let state = AppState::default();

    assert_eq!(state.live_status, LiveStatus::Idle);
}

#[test]
fn toggle_theme_cycles_correctly() {
    let mut state = AppState::default(); // Starts at Light
    assert_eq!(state.selected_theme, ThemePreference::Light);

    state.toggle_theme();
    assert_eq!(state.selected_theme, ThemePreference::Dark);

    state.toggle_theme();
    assert_eq!(state.selected_theme, ThemePreference::Blue);

    state.toggle_theme();
    assert_eq!(state.selected_theme, ThemePreference::Light);
}

#[test]
fn desktop_assets_include_app_stylesheet() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")).unwrap();

    assert!(source.contains("dx-components-theme.css"));
    assert!(source.contains("custom-themes.css"));
}

#[test]
fn app_source_keeps_dioxus_theme_and_applies_data_theme() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")).unwrap();

    assert!(source.contains("dx-components-theme.css"));
    assert!(source.contains("document::eval"));
    assert!(source.contains("document.documentElement.setAttribute('data-theme'"));
}

#[test]
fn app_stylesheet_sets_desktop_root_surface() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/dx-components-theme.css"
    ))
    .unwrap();

    assert!(source.contains(":root") || source.contains("html") || source.contains("body"));
}
