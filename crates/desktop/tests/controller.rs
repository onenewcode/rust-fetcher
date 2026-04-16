use desktop::controller::Controller;
use tempfile::TempDir;

#[tokio::test]
async fn controller_exposes_service_event_receiver_once() {
    let temp_dir = TempDir::new().unwrap();
    let mut controller = Controller::new(temp_dir.path().to_path_buf());

    assert!(controller.take_event_receiver().is_some());
    assert!(controller.take_event_receiver().is_none());
}
