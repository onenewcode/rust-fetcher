use im::proto::request::{Request, request_body};
use im::request_builder::{apply_message_text, apply_receiver_id, build_request_body};
use im::request_template::load_template_proto;
use serde_json::Value;

#[test]
fn apply_receiver_id_replaces_the_target_segment() {
    let updated = apply_receiver_id("0:1:old-user:3", "new-user").unwrap();
    assert_eq!(updated, "0:1:new-user:3");
}

#[test]
fn apply_message_text_preserves_existing_fields() {
    let content = r#"{"mentionUsers":["u1"],"aweType":700,"richTextInfos":["x"],"text":"hello"}"#;

    let updated = apply_message_text(content, "updated").unwrap();
    let value: Value = serde_json::from_str(&updated).unwrap();

    assert_eq!(value["text"], "updated");
    assert_eq!(value["aweType"], 700);
    assert_eq!(value["mentionUsers"], serde_json::json!(["u1"]));
    assert_eq!(value["richTextInfos"], serde_json::json!(["x"]));
}

#[test]
fn build_request_body_updates_conversation_text_and_ext_fields() {
    let mut request: Request = load_template_proto().unwrap();
    let config = im::models::IMSendConfig {
        cookie: "cookie=value".to_string(),
        timeout: 30,
        receiver_id: Some("receiver-42".to_string()),
        conversation_id: Some("0:1:conversation-99:3".to_string()),
        message_text: Some("updated message".to_string()),
    };

    let body = build_request_body(&mut request, &config).unwrap();
    assert!(!body.is_empty());
    assert!(request.sequence_id > 0);

    let send_body = match request.body.as_ref().and_then(|body| body.body.as_ref()) {
        Some(request_body::Body::SendMessageBody(send_body)) => send_body,
        other => panic!("unexpected request body: {other:?}"),
    };

    assert_eq!(send_body.conversation_id, "0:1:conversation-99:3");

    let value: Value = serde_json::from_str(&send_body.content).unwrap();
    assert_eq!(value["text"], "updated message");

    let client_message_id = send_body
        .ext
        .iter()
        .find(|ext| ext.key == "s:client_message_id")
        .map(|ext| ext.value.as_str())
        .unwrap();
    assert_eq!(send_body.client_message_id, client_message_id);
    assert!(
        send_body
            .ext
            .iter()
            .any(|ext| ext.key == "s:stime" && !ext.value.is_empty())
    );
}
