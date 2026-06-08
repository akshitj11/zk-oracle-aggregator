//! Adversarial / negative-path tests for M0–M2 security invariants.
//! See docs/security/adversarial-vectors.md.

use oracle_core::{
    aggregate, parse_response, weighted_median, AggregationResult, Outcome,
    ParseError, SourceResponse,
};

fn make_response(
    source_id: &str,
    outcome: Outcome,
    confidence: f64,
) -> SourceResponse {
    SourceResponse {
        source_id: source_id.to_owned(),
        outcome,
        confidence,
        fetched_at: 1,
        raw_hash: [0u8; 32],
    }
}

#[test]
fn parse_rejects_invalid_json() {
    let err = parse_response("x", "not json", [0; 32], 0).unwrap_err();
    assert!(matches!(err, ParseError::InvalidJson(_)));
}

#[test]
fn parse_rejects_negative_confidence() {
    let body = r#"{"outcome":"YES","confidence":-0.1}"#;
    let err = parse_response("x", body, [0; 32], 0).unwrap_err();
    assert_eq!(err, ParseError::InvalidConfidence(-0.1));
}

#[test]
fn empty_aggregate_disputed() {
    let result = aggregate(&[]);
    assert_eq!(
        result,
        AggregationResult {
            outcome: Outcome::Unknown,
            confidence: rust_decimal::Decimal::ZERO,
            source_count: 0,
            agreement_ratio: rust_decimal::Decimal::ZERO,
            disputed: true,
        }
    );
}

#[test]
fn minority_outcomes_filtered_from_consensus() {
    let responses = vec![
        make_response("a", Outcome::Yes, 0.9),
        make_response("b", Outcome::Yes, 0.85),
        make_response("c", Outcome::Yes, 0.88),
        make_response("d", Outcome::No, 0.3),
    ];
    let result = aggregate(&responses);
    assert!(!result.disputed);
    assert_eq!(result.source_count, 3);
    assert_eq!(result.outcome, Outcome::Yes);
}

#[test]
fn tie_weight_returns_no() {
    let responses = vec![
        make_response("a", Outcome::Yes, 0.5),
        make_response("b", Outcome::No, 0.5),
    ];
    assert_eq!(weighted_median(&responses), Outcome::No);
}

#[test]
fn unknown_only_returns_unknown() {
    let responses = vec![
        make_response("a", Outcome::Unknown, 0.9),
        make_response("b", Outcome::Unknown, 0.8),
    ];
    assert_eq!(weighted_median(&responses), Outcome::Unknown);
}

#[test]
fn low_agreement_marks_disputed() {
    // 29 Yes + 29 No fail the 30% agreement bar; 42 Unknown remain → 42% agreement.
    let mut responses = Vec::with_capacity(100);
    for i in 0..29 {
        responses.push(make_response(&format!("y{i}"), Outcome::Yes, 0.9));
    }
    for i in 0..29 {
        responses.push(make_response(&format!("n{i}"), Outcome::No, 0.9));
    }
    for i in 0..42 {
        responses.push(make_response(&format!("u{i}"), Outcome::Unknown, 0.5));
    }
    let result = aggregate(&responses);
    assert!(result.disputed);
    assert_eq!(result.source_count, 42);
}
