use live::fetcher::connection;

#[test]
fn connection_stops_on_close_frame() {
    assert!(connection::should_stop_after_close());
}
