use service::open::{export_dir, log_dir};
use std::path::Path;

#[test]
fn log_dir_is_project_logs_directory() {
    let root = Path::new("/tmp/project-root");
    assert_eq!(log_dir(root), root.join("logs"));
}

#[test]
fn export_dir_is_project_exports_directory() {
    let root = Path::new("/tmp/project-root");
    assert_eq!(export_dir(root), root.join("exports"));
}
