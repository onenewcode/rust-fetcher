use dioxus::prelude::*;
use service::config::{AppConfig, ThemePreference};
use service::event::{
    BulkPhase, BulkProgress, ImBulkError, LiveEvent, LiveLog, LiveStatus, LogLevel,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub config: AppConfig,
    pub selected_theme: ThemePreference,
    pub live_status: LiveStatus,
    pub error_message: Option<String>,
    pub message_count: u64,
    pub last_event_at: Option<String>,

    // Bulk IM State
    pub bulk_csv_path: Option<std::path::PathBuf>,
    pub is_bulk_sending: bool,
    pub bulk_progress: usize,
    pub bulk_total: usize,
    pub bulk_status: Option<BulkPhase>,
}

impl AppState {
    pub fn is_running(&self) -> bool {
        matches!(self.live_status, LiveStatus::Running | LiveStatus::Starting)
    }

    pub fn new() -> Self {
        Self::from_config(AppConfig::default())
    }

    pub fn from_config(config: AppConfig) -> Self {
        let selected_theme = config.theme;

        Self {
            config,
            selected_theme,
            live_status: LiveStatus::Idle,
            error_message: None,
            message_count: 0,
            last_event_at: None,
            bulk_csv_path: None,
            is_bulk_sending: false,
            bulk_progress: 0,
            bulk_total: 0,
            bulk_status: None,
        }
    }

    pub fn toggle_theme(&mut self) {
        self.selected_theme = self.selected_theme.next();
    }

    pub fn bulk_status_text(&self) -> Option<String> {
        self.bulk_status
            .as_ref()
            .map(|phase| format_bulk_status(phase, self.bulk_progress, self.bulk_total))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn reduce_event(state: &mut AppState, event: service::event::LiveEvent) {
    match event {
        LiveEvent::StatusChanged(status) => {
            state.live_status = status;
            if status == LiveStatus::Running {
                state.error_message = None;
            }
        }
        LiveEvent::Log(LiveLog { level, message }) => {
            if level == LogLevel::Error {
                state.error_message = Some(message);
            }
        }
        LiveEvent::Metrics {
            message_count,
            last_event_at,
        } => {
            state.message_count = message_count;
            state.last_event_at = last_event_at;
        }
        LiveEvent::ImBulkProgress(BulkProgress {
            progress,
            total,
            phase,
        }) => {
            state.bulk_progress = progress;
            state.bulk_total = total;
            state.bulk_status = Some(phase);
        }
        LiveEvent::ImBulkCompleted {
            success_count,
            fail_count,
        } => {
            state.is_bulk_sending = false;
            state.bulk_status = Some(BulkPhase::Completed {
                success_count,
                fail_count,
            });
        }
        LiveEvent::ImBulkError(ImBulkError { message }) => {
            state.is_bulk_sending = false;
            state.error_message = Some(message);
        }
    }
}

fn format_bulk_status(phase: &BulkPhase, progress: usize, total: usize) -> String {
    match phase {
        BulkPhase::Starting => format!("Starting bulk send to {total} users"),
        BulkPhase::Sending { user_name } => {
            format!("Sending to {user_name} ({progress}/{total})")
        }
        BulkPhase::Waiting { seconds } => format!("Waiting {seconds}s before next..."),
        BulkPhase::Completed {
            success_count,
            fail_count,
        } => {
            format!("Completed: {success_count} success, {fail_count} failed")
        }
    }
}

pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}
