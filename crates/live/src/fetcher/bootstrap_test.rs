use live::fetcher::bootstrap;

#[test]
fn bootstrap_uses_cached_room_id_when_available() {
    let room_id = bootstrap::cached_room_id(Some("room-42".to_string())).unwrap();
    assert_eq!(room_id, Some("room-42".to_string()));
}

#[test]
fn bootstrap_skips_cached_room_id_when_missing() {
    let room_id = bootstrap::cached_room_id(None).unwrap();
    assert_eq!(room_id, None);
}
