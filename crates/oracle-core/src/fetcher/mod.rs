//! Concurrent HTTP fetching from oracle data sources.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::{sleep, timeout};

/// Binary outcome from a data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Outcome {
    /// Affirmative resolution.
    Yes,
    /// Negative resolution.
    No,
    /// Source could not determine an outcome.
    Unknown,
}

/// Normalized response from one oracle source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceResponse {
    /// Stable identifier, e.g. `ap-news`.
    pub source_id: String,
    /// Parsed outcome.
    pub outcome: Outcome,
    /// Confidence in `0.0..=1.0`.
    pub confidence: f64,
    /// Unix timestamp when the response was fetched.
    pub fetched_at: u64,
    /// Blake3 hash of the raw HTTP body.
    pub raw_hash: [u8; 32],
}

#[derive(Debug, Deserialize)]
struct ApiPayload {
    outcome: Outcome,
    confidence: f64,
}

/// Errors while parsing a source body.
#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("invalid json: {0}")]
    InvalidJson(String),
    #[error("confidence {0} out of range 0.0..=1.0")]
    InvalidConfidence(f64),
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Parse a JSON API body into a [`SourceResponse`].
pub fn parse_response(
    source_id: &str,
    body: &str,
    raw_hash: [u8; 32],
    fetched_at: u64,
) -> Result<SourceResponse, ParseError> {
    let payload: ApiPayload = serde_json::from_str(body)
        .map_err(|e| ParseError::InvalidJson(e.to_string()))?;

    if !(0.0..=1.0).contains(&payload.confidence) {
        return Err(ParseError::InvalidConfidence(payload.confidence));
    }

    Ok(SourceResponse {
        source_id: source_id.to_owned(),
        outcome: payload.outcome,
        confidence: payload.confidence,
        fetched_at,
        raw_hash,
    })
}

/// Fetch one source with timeout and retries.
pub async fn fetch_source(
    client: &Client,
    source_url: &str,
    source_id: &str,
    retries: u8,
) -> Option<SourceResponse> {
    for attempt in 0..=retries {
        let result =
            timeout(Duration::from_secs(5), client.get(source_url).send())
                .await;

        match result {
            Ok(Ok(resp)) if resp.status().is_success() => {
                let body = resp.text().await.ok()?;
                let hash = *blake3::hash(body.as_bytes()).as_bytes();
                let fetched_at = unix_now();
                return parse_response(source_id, &body, hash, fetched_at).ok();
            }
            _ => {
                if attempt < retries {
                    let delay_ms =
                        200_u64.saturating_mul(u64::from(attempt) + 1);
                    sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
    None
}

/// Fetch all configured sources concurrently.
pub async fn fetch_all_sources(
    client: &Client,
    sources: &[(String, String)],
) -> Vec<SourceResponse> {
    let futures = sources
        .iter()
        .map(|(id, url)| fetch_source(client, url, id, 2));

    join_all(futures).await.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn parse_yes_outcome() {
        let body = r#"{"outcome":"YES","confidence":0.95}"#;
        let hash = [0u8; 32];
        let resp =
            parse_response("ap-news", body, hash, 1_700_000_000).unwrap();
        assert_eq!(resp.outcome, Outcome::Yes);
        assert!((resp.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(resp.source_id, "ap-news");
    }

    #[test]
    fn parse_rejects_invalid_confidence() {
        let body = r#"{"outcome":"YES","confidence":1.5}"#;
        let err = parse_response("x", body, [0; 32], 0).unwrap_err();
        assert_eq!(err, ParseError::InvalidConfidence(1.5));
    }

    #[tokio::test]
    async fn fetch_all_sources_skips_failed() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ok1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "outcome": "YES",
                    "confidence": 0.9
                }),
            ))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/ok2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "outcome": "NO",
                    "confidence": 0.85
                }),
            ))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/fail"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = Client::new();
        let sources = vec![
            ("src-a".to_owned(), format!("{}/ok1", server.uri())),
            ("src-b".to_owned(), format!("{}/ok2", server.uri())),
            ("src-c".to_owned(), format!("{}/fail", server.uri())),
        ];

        let responses = fetch_all_sources(&client, &sources).await;
        assert_eq!(responses.len(), 2);
        assert!(responses.iter().any(|r| r.source_id == "src-a"));
        assert!(responses.iter().any(|r| r.source_id == "src-b"));
    }
}
