#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveStatus {
    Idle,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulkPhase {
    Starting,
    Sending {
        user_name: String,
    },
    Waiting {
        seconds: u64,
    },
    Completed {
        success_count: usize,
        fail_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkProgress {
    pub progress: usize,
    pub total: usize,
    pub phase: BulkPhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImBulkError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveLog {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveEvent {
    StatusChanged(LiveStatus),
    Log(LiveLog),
    Metrics {
        message_count: u64,
        last_event_at: Option<String>,
    },
    ImBulkProgress(BulkProgress),
    ImBulkCompleted {
        success_count: usize,
        fail_count: usize,
    },
    ImBulkError(ImBulkError),
}
