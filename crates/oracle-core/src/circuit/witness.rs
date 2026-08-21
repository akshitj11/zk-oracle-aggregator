//! Build [`super::OracleCircuit`] witnesses from fetcher and aggregator output.

use std::collections::HashSet;

use ark_bn254::Fr;
use thiserror::Error;

use crate::aggregator::{aggregate, remove_outliers, AggregationResult};
use crate::fetcher::{Outcome, SourceResponse};
use crate::MAX_SOURCES;

use super::field::{confidence_to_field, outcome_to_field, CONFIDENCE_SCALE};
use super::oracle_circuit::{OracleCircuit, MAJORITY_MARGIN_BITS};

/// Errors building a circuit witness from live source data.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WitnessError {
    #[error("too many sources: {0} > MAX_SOURCES {MAX_SOURCES}")]
    TooManySources(usize),
    #[error("aggregation is disputed; cannot prove")]
    Disputed,
    #[error("witness outcome does not match aggregation result")]
    AggregationMismatch,
}

/// Build a padded circuit and the aggregation result used for public inputs.
pub fn build_witness(
    responses: &[SourceResponse],
) -> Result<(OracleCircuit, AggregationResult), WitnessError> {
    if responses.len() > MAX_SOURCES {
        return Err(WitnessError::TooManySources(responses.len()));
    }

    let result = aggregate(responses);
    if result.disputed {
        return Err(WitnessError::Disputed);
    }

    let filtered = remove_outliers(responses, 0.70);
    let included_ids: HashSet<_> =
        filtered.iter().map(|r| r.source_id.as_str()).collect();

    let mut source_outcomes = Vec::with_capacity(MAX_SOURCES);
    let mut source_weights = Vec::with_capacity(MAX_SOURCES);
    let mut source_included = Vec::with_capacity(MAX_SOURCES);

    for response in responses {
        let included = included_ids.contains(response.source_id.as_str());
        source_outcomes.push(Some(outcome_to_field(response.outcome)));
        source_weights.push(Some(confidence_to_field(response.confidence)));
        source_included.push(Some(Fr::from(u64::from(included))));
    }

    while source_outcomes.len() < MAX_SOURCES {
        source_outcomes.push(Some(Fr::from(0u64)));
        source_weights.push(Some(Fr::from(0u64)));
        source_included.push(Some(Fr::from(0u64)));
    }

    let final_outcome = match result.outcome {
        Outcome::Yes => Some(Fr::from(1u64)),
        Outcome::No => Some(Fr::from(0u64)),
        Outcome::Unknown => return Err(WitnessError::AggregationMismatch),
    };

    let source_count = Some(Fr::from(result.source_count as u64));

    let (yes_weight, total_weight) = scaled_weights(responses, &included_ids);
    let outcome_yes = result.outcome == Outcome::Yes;
    let margin = majority_margin(yes_weight, total_weight, outcome_yes);
    let majority_margin_bits = margin_to_bits(margin);

    let circuit = OracleCircuit {
        source_outcomes,
        source_weights,
        source_included,
        majority_margin_bits,
        final_outcome,
        source_count,
    };

    let expected_yes = result.outcome == Outcome::Yes;
    let circuit_yes = circuit.final_outcome == Some(Fr::from(1u64));
    if expected_yes != circuit_yes {
        return Err(WitnessError::AggregationMismatch);
    }

    Ok((circuit, result))
}

/// Source ids that survive outlier removal at the default threshold.
pub fn included_source_ids(responses: &[SourceResponse]) -> HashSet<String> {
    remove_outliers(responses, 0.70)
        .into_iter()
        .map(|r| r.source_id)
        .collect()
}

/// Blake3 hash over included source `raw_hash` values (Z5 commitment).
pub fn agreement_hash(
    responses: &[SourceResponse],
    included_ids: &HashSet<&str>,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for response in responses {
        if included_ids.contains(response.source_id.as_str()) {
            hasher.update(&response.raw_hash);
        }
    }
    *hasher.finalize().as_bytes()
}

/// Empty padded circuit for trusted setup (all-zero witnesses).
pub fn padded_empty_circuit() -> OracleCircuit {
    OracleCircuit {
        source_outcomes: vec![Some(Fr::from(0u64)); MAX_SOURCES],
        source_weights: vec![Some(Fr::from(0u64)); MAX_SOURCES],
        source_included: vec![Some(Fr::from(0u64)); MAX_SOURCES],
        majority_margin_bits: vec![Some(Fr::from(0u64)); MAJORITY_MARGIN_BITS],
        final_outcome: Some(Fr::from(0u64)),
        source_count: Some(Fr::from(0u64)),
    }
}

fn scaled_weights(
    responses: &[SourceResponse],
    included_ids: &HashSet<&str>,
) -> (u64, u64) {
    let mut yes = 0u64;
    let mut total = 0u64;
    for response in responses {
        if !included_ids.contains(response.source_id.as_str()) {
            continue;
        }
        let weight = (response.confidence.clamp(0.0, 1.0)
            * CONFIDENCE_SCALE as f64)
            .round() as u64;
        total = total.saturating_add(weight);
        if response.outcome == Outcome::Yes {
            yes = yes.saturating_add(weight);
        }
    }
    (yes, total)
}

fn majority_margin(
    yes_weight: u64,
    total_weight: u64,
    outcome_yes: bool,
) -> u64 {
    let two_yes = yes_weight.saturating_mul(2);
    if outcome_yes {
        two_yes.saturating_sub(total_weight).saturating_sub(1)
    } else {
        total_weight.saturating_sub(two_yes)
    }
}

fn margin_to_bits(margin: u64) -> Vec<Option<Fr>> {
    (0..MAJORITY_MARGIN_BITS)
        .map(|bit| Some(Fr::from((margin >> bit) & 1)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetcher::SourceResponse;

    fn response(id: &str, outcome: Outcome, confidence: f64) -> SourceResponse {
        SourceResponse {
            source_id: id.to_owned(),
            outcome,
            confidence,
            fetched_at: 1,
            raw_hash: [0u8; 32],
        }
    }

    #[test]
    fn build_witness_matches_aggregate() {
        let responses = vec![
            response("a", Outcome::Yes, 0.9),
            response("b", Outcome::Yes, 0.85),
            response("c", Outcome::No, 0.2),
        ];
        let (circuit, result) = build_witness(&responses).unwrap();
        assert!(!result.disputed);
        assert_eq!(result.outcome, Outcome::Yes);
        assert_eq!(circuit.source_count, Some(Fr::from(3u64)));
    }

    #[test]
    fn included_source_ids_drops_minority() {
        let responses = vec![
            response("a", Outcome::Yes, 0.9),
            response("b", Outcome::Yes, 0.9),
            response("c", Outcome::Yes, 0.9),
            response("d", Outcome::No, 0.9),
        ];
        let ids = included_source_ids(&responses);
        assert_eq!(ids.len(), 3);
        assert!(!ids.contains("d"));
    }

    #[test]
    fn disputed_returns_error() {
        assert!(matches!(build_witness(&[]), Err(WitnessError::Disputed)));
    }
}
