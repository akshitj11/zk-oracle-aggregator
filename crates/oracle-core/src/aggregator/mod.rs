//! Aggregation engine: outlier removal, weighted median, consensus.

#![allow(clippy::cast_precision_loss)] // ratios from small source counts; inputs use f64 confidence

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::fetcher::{Outcome, SourceResponse};

/// Result of aggregating multiple source responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregationResult {
    pub outcome: Outcome,
    pub confidence: Decimal,
    pub source_count: usize,
    pub agreement_ratio: Decimal,
    pub disputed: bool,
}

/// Remove sources that disagree strongly with the majority.
pub fn remove_outliers(
    responses: &[SourceResponse],
    deviation_threshold: f64,
) -> Vec<SourceResponse> {
    if responses.is_empty() {
        return Vec::new();
    }

    let total = responses.len() as f64;
    let min_agreement = 1.0 - deviation_threshold;

    responses
        .iter()
        .filter(|r| {
            let agreeing = responses
                .iter()
                .filter(|other| other.outcome == r.outcome)
                .count() as f64;
            agreeing / total >= min_agreement
        })
        .cloned()
        .collect()
}

/// Weighted median: compare total confidence weight for Yes vs No.
pub fn weighted_median(responses: &[SourceResponse]) -> Outcome {
    let yes_weight: f64 = responses
        .iter()
        .filter(|r| r.outcome == Outcome::Yes)
        .map(|r| r.confidence)
        .sum();

    let no_weight: f64 = responses
        .iter()
        .filter(|r| r.outcome == Outcome::No)
        .map(|r| r.confidence)
        .sum();

    let total_weight = yes_weight + no_weight;
    if total_weight == 0.0 {
        return Outcome::Unknown;
    }

    if yes_weight / total_weight > 0.5 {
        Outcome::Yes
    } else {
        Outcome::No
    }
}

/// Full aggregation pipeline.
pub fn aggregate(responses: &[SourceResponse]) -> AggregationResult {
    if responses.is_empty() {
        return AggregationResult {
            outcome: Outcome::Unknown,
            confidence: Decimal::ZERO,
            source_count: 0,
            agreement_ratio: Decimal::ZERO,
            disputed: true,
        };
    }

    let filtered = remove_outliers(responses, 0.70);
    let agreement_ratio_f = filtered.len() as f64 / responses.len() as f64;
    let disputed = agreement_ratio_f < 0.60 || filtered.is_empty();

    if filtered.is_empty() {
        return AggregationResult {
            outcome: Outcome::Unknown,
            confidence: Decimal::ZERO,
            source_count: 0,
            agreement_ratio: decimal_from_f64(agreement_ratio_f),
            disputed: true,
        };
    }

    let filtered_len = filtered.len();
    let outcome = weighted_median(&filtered);

    let confidence_f = filtered
        .iter()
        .filter(|r| r.outcome == outcome)
        .map(|r| r.confidence)
        .sum::<f64>()
        / filtered_len as f64;

    AggregationResult {
        outcome,
        confidence: decimal_from_f64(confidence_f),
        source_count: filtered.len(),
        agreement_ratio: decimal_from_f64(agreement_ratio_f),
        disputed,
    }
}

fn decimal_from_f64(value: f64) -> Decimal {
    Decimal::from_f64_retain(value).unwrap_or(Decimal::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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
    fn outlier_removal_drops_minority() {
        let responses = vec![
            make_response("reuters", Outcome::Yes, 0.9),
            make_response("ap-news", Outcome::Yes, 0.85),
            make_response("bbc", Outcome::Yes, 0.88),
            make_response("bad-src", Outcome::No, 0.3),
        ];

        let filtered = remove_outliers(&responses, 0.70);
        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|r| r.outcome == Outcome::Yes));
    }

    #[test]
    fn aggregation_is_deterministic() {
        let responses = vec![
            make_response("a", Outcome::Yes, 0.9),
            make_response("b", Outcome::Yes, 0.8),
            make_response("c", Outcome::No, 0.2),
        ];

        let r1 = aggregate(&responses);
        let r2 = aggregate(&responses);
        assert_eq!(r1, r2);
    }

    #[test]
    fn aggregate_majority_yes() {
        let responses = vec![
            make_response("a", Outcome::Yes, 0.9),
            make_response("b", Outcome::Yes, 0.85),
            make_response("c", Outcome::Yes, 0.88),
            make_response("d", Outcome::No, 0.3),
        ];

        let result = aggregate(&responses);
        assert_eq!(result.outcome, Outcome::Yes);
        assert!(!result.disputed);
        assert_eq!(result.source_count, 3);
    }

    proptest! {
        #[test]
        fn weighted_median_never_panics(
            outcomes in prop::collection::vec(0u8..=1, 1..20),
            weights in prop::collection::vec(0.0f64..=1.0, 1..20),
        ) {
            let len = outcomes.len().min(weights.len());
            let responses: Vec<_> = outcomes
                .into_iter()
                .zip(weights)
                .take(len)
                .map(|(o, w)| make_response(
                    "src",
                    if o == 1 { Outcome::Yes } else { Outcome::No },
                    w,
                ))
                .collect();

            let _ = weighted_median(&responses);
        }
    }
}
