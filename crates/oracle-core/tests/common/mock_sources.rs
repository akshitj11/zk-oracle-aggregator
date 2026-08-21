//! Wiremock fixture: three independent source endpoints for integration tests.

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Three mock HTTP sources mounted on one wiremock server.
pub struct MockSourceCluster {
    /// Running mock server; keep alive for the duration of the test.
    #[allow(dead_code)]
    pub server: MockServer,
    /// `(source_id, url)` pairs ready for `fetch_all_sources`.
    pub sources: Vec<(String, String)>,
}

impl MockSourceCluster {
    /// Mount three JSON sources: two YES, one NO (minority outlier).
    pub async fn three_source_consensus() -> Self {
        let server = MockServer::start().await;
        let base = server.uri();

        Mock::given(method("GET"))
            .and(path("/ap"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "outcome": "YES",
                    "confidence": 0.92
                }),
            ))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/reuters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "outcome": "YES",
                    "confidence": 0.88
                }),
            ))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/bbc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "outcome": "NO",
                    "confidence": 0.35
                }),
            ))
            .mount(&server)
            .await;

        let sources = vec![
            ("ap-news".to_owned(), format!("{base}/ap")),
            ("reuters".to_owned(), format!("{base}/reuters")),
            ("bbc".to_owned(), format!("{base}/bbc")),
        ];

        Self { server, sources }
    }
}
