use im::error::IMError;
use im::models::SendResult;
use im::response::{decode_into_result, validate_result};
use prost::Message;

#[test]
fn decode_into_result_keeps_proto_and_empty_preview_for_valid_bytes() {
    let proto = im::proto::response::Response {
        cmd: 0,
        sequence_id: 42,
        inbox_type: 0,
        message: "OK".to_string(),
        status_code: 0,
        body: None,
    };
    let mut bytes = Vec::new();
    proto.encode(&mut bytes).unwrap();

    let result = decode_into_result(
        "https://example.com/send".to_string(),
        200,
        "application/x-protobuf".to_string(),
        &bytes,
    );

    assert_eq!(result.request_url, "https://example.com/send");
    assert_eq!(result.http_status, 200);
    assert_eq!(result.content_type, "application/x-protobuf");
    assert_eq!(result.response_size, bytes.len());
    assert_eq!(result.response_text_preview, "");
    assert_eq!(result.response_proto.unwrap().message, "OK");
}

#[test]
fn decode_into_result_captures_preview_when_proto_decode_fails() {
    let bytes = b"not protobuf response";

    let result = decode_into_result(
        "https://example.com/send".to_string(),
        502,
        "text/plain".to_string(),
        bytes,
    );

    assert_eq!(result.response_size, bytes.len());
    assert!(result.response_proto.is_none());
    assert_eq!(result.response_text_preview, "not protobuf response");
}

#[test]
fn validate_result_rejects_non_200_status() {
    let result = SendResult {
        request_url: "https://example.com".to_string(),
        http_status: 500,
        content_type: "application/x-protobuf".to_string(),
        response_size: 0,
        response_proto: None,
        response_text_preview: String::new(),
    };

    let error = validate_result(&result).unwrap_err();
    match error {
        IMError::Api(message, code) => {
            assert_eq!(message, "HTTP status not OK: 500");
            assert_eq!(code, 0);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn validate_result_requires_decoded_proto() {
    let result = SendResult {
        request_url: "https://example.com".to_string(),
        http_status: 200,
        content_type: "application/x-protobuf".to_string(),
        response_size: 0,
        response_proto: None,
        response_text_preview: String::new(),
    };

    let error = validate_result(&result).unwrap_err();
    match error {
        IMError::Message(message) => {
            assert_eq!(message, "Response protobuf parse failed");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
