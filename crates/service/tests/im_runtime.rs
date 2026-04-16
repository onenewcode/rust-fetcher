use service::event::{BulkPhase, BulkProgress, ImBulkError, LiveEvent, LiveLog, LogLevel};

#[test]
fn live_log_wraps_level_and_message() {
    let event = LiveEvent::Log(LiveLog {
        level: LogLevel::Error,
        message: "boom".to_string(),
    });

    match event {
        LiveEvent::Log(log) => {
            assert_eq!(log.level, LogLevel::Error);
            assert_eq!(log.message, "boom");
        }
        _ => panic!("unexpected event variant"),
    }
}

#[test]
fn im_bulk_progress_is_structured() {
    let event = LiveEvent::ImBulkProgress(BulkProgress {
        progress: 1,
        total: 2,
        phase: BulkPhase::Sending {
            user_name: "alice".to_string(),
        },
    });

    match event {
        LiveEvent::ImBulkProgress(progress) => {
            assert_eq!(progress.progress, 1);
            assert_eq!(progress.total, 2);
            assert_eq!(
                progress.phase,
                BulkPhase::Sending {
                    user_name: "alice".to_string(),
                }
            );
        }
        _ => panic!("unexpected event variant"),
    }
}

#[test]
fn im_bulk_error_wraps_message_in_structured_payload() {
    let event = LiveEvent::ImBulkError(ImBulkError {
        message: "boom".to_string(),
    });

    match event {
        LiveEvent::ImBulkError(error) => {
            assert_eq!(error.message, "boom");
        }
        _ => panic!("unexpected event variant"),
    }
}
