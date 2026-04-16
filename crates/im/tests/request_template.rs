use im::proto::request::{Request, request_body};
use im::request_template::load_template_proto;

#[test]
fn load_template_proto_contains_send_message_body() {
    let request = load_template_proto().unwrap();

    assert!(matches!(
        request.body.as_ref().and_then(|body| body.body.as_ref()),
        Some(request_body::Body::SendMessageBody(_))
    ));
}

#[test]
fn load_template_proto_populates_sequence_id() {
    let request: Request = load_template_proto().unwrap();
    assert!(request.sequence_id > 0);
}
