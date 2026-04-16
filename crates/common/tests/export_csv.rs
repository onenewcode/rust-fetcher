use common::export::{CommentUserCsvExporter, CommentUserRecord};

#[test]
fn exporter_writes_header_once_and_reads_records_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut exporter = CommentUserCsvExporter::new(dir.path(), "room-1");

    exporter
        .append_comment(&CommentUserRecord {
            comment_time: "1".to_string(),
            comment_content: "hello,world".to_string(),
            user_id: "u1".to_string(),
            user_name: "alice".to_string(),
            profile_url: "https://example.com/u1".to_string(),
        })
        .unwrap();

    let records = CommentUserCsvExporter::read_comments(exporter.path()).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].comment_content, "hello,world");
}
