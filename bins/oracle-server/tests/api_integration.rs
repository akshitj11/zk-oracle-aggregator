//! API integration tests (require DATABASE_URL).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oracle_core::prover::OracleProver;
use oracle_core::storage::OracleStore;
use oracle_server::build_router;
use oracle_server::state::AppState;
use reqwest::Client;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn test_state(
    api_key: Option<String>,
    sources: Vec<(String, String)>,
) -> AppState {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let store = OracleStore::connect(&url).await.expect("connect");
    let prover = OracleProver::generate_keys().expect("keys");
    let verifier = prover.verifier();
    AppState::new(store, prover, verifier, Client::new(), sources, api_key)
}

async fn mock_consensus_sources() -> (MockServer, Vec<(String, String)>) {
    let server = MockServer::start().await;
    let base = server.uri();

    for (p, outcome, confidence) in
        [("/a", "YES", 0.9), ("/b", "YES", 0.85), ("/c", "NO", 0.2)]
    {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "outcome": outcome,
                    "confidence": confidence
                }),
            ))
            .mount(&server)
            .await;
    }

    let sources = vec![
        ("a".to_string(), format!("{base}/a")),
        ("b".to_string(), format!("{base}/b")),
        ("c".to_string(), format!("{base}/c")),
    ];
    (server, sources)
}

async fn mock_disputed_sources() -> (MockServer, Vec<(String, String)>) {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/down"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let sources = vec![("down".to_string(), format!("{base}/down"))];
    (server, sources)
}

#[tokio::test]
async fn health_requires_no_auth() {
    let (_mock, sources) = mock_consensus_sources().await;
    let state = test_state(None, sources).await;
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn resolve_full_pipeline() {
    let (_mock, sources) = mock_consensus_sources().await;
    let state = test_state(None, sources).await;
    let app = build_router(state);

    let body = serde_json::json!({ "market_id": "aa".repeat(32) });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resolve")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn resolve_disputed_returns_409() {
    let (_mock, sources) = mock_disputed_sources().await;
    let state = test_state(None, sources).await;
    let app = build_router(state);

    let body = serde_json::json!({ "market_id": "bb".repeat(32) });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resolve")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn auth_rejects_missing_key() {
    let (_mock, sources) = mock_consensus_sources().await;
    let state = test_state(Some("secret-key".to_string()), sources).await;
    let app = build_router(state);

    let body = serde_json::json!({ "market_id": "cc".repeat(32) });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resolve")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn verify_stored_proof_after_resolve() {
    let (_mock, sources) = mock_consensus_sources().await;
    let state = test_state(None, sources).await;
    let market_hex = "dd".repeat(32);

    let app = build_router(state.clone());
    let body = serde_json::json!({ "market_id": market_hex });
    let resolve = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resolve")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve.status(), StatusCode::OK);

    let app = build_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/verify/{market_hex}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let ok: bool = serde_json::from_slice(&body).unwrap();
    assert!(ok);
}

#[tokio::test]
async fn rate_limit_returns_429() {
    let (_mock, sources) = mock_consensus_sources().await;
    let state = test_state(None, sources).await;
    let app = build_router(state);

    let body = serde_json::json!({ "market_id": "ee".repeat(32) });
    let mut saw_429 = false;
    for _ in 0..15 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            saw_429 = true;
            break;
        }
    }
    assert!(saw_429);
}
