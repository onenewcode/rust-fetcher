use common::ids::{CookieString, RoomId};

#[test]
fn room_id_rejects_blank_values() {
    assert!(RoomId::new("   ").is_err());
}

#[test]
fn cookie_string_rejects_blank_values() {
    assert!(CookieString::new("\n\t").is_err());
}
