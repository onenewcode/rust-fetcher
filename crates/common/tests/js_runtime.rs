use common::js::{SharedJsRuntime, SignRequest, WebsocketSignatureRequest};

#[tokio::test]
async fn js_runtime_executes_sign_requests() {
    let runtime = SharedJsRuntime::start_for_test().unwrap();

    let result = runtime
        .sign(SignRequest::test("return input + '-signed';", "abc"))
        .await
        .unwrap();

    assert_eq!(result.output, "abc-signed");
}

#[tokio::test]
async fn js_runtime_executes_typed_websocket_sign_requests() {
    let runtime = SharedJsRuntime::start_for_test().unwrap();

    let result = runtime
        .sign_websocket(WebsocketSignatureRequest {
            script: "function get_sign(input) { return input + '-ws'; }".to_string(),
            md5_stub: "abc".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result, "abc-ws");
}
