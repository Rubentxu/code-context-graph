use axum::{http::{Request, StatusCode}, body::{Body, to_bytes}};
use code_context_graph_api::router;
use serde_json::json;
use tower::util::ServiceExt; // for `oneshot`

#[tokio::test]
async fn health_returns_ok() {
    let app = router();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v, json!({"status":"ok"}));
}

#[tokio::test]
async fn query_endpoint_basic() {
    let app = router();

    let req_body = json!({
        "question": "who calls X?",
        "version": "latest"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/query")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["version_info"]["version_id"], json!("latest"));
}

#[tokio::test]
async fn llm_context_endpoint_basic() {
    let app = router();

    let req_body = json!({
        "focus": {"type": "UseCase", "id": "UCS-payment-flow"},
        "token_budget": 8000
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/query/llm-context")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["focus"]["type"], json!("UseCase"));
    assert_eq!(v["limits"]["token_budget"], json!(8000));
}

#[tokio::test]
async fn explain_change_endpoint_basic() {
    let app = router();

    let req_body = json!({
        "from_version": "v1",
        "to_version": "v2"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/query/explain-change")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v["summary"].is_object());
}
