//! High-level prove API over fetcher input.

use std::collections::HashSet;

use crate::aggregator::AggregationResult;
use crate::circuit::{agreement_hash, build_witness, included_source_ids};
use crate::fetcher::SourceResponse;

use super::engine::{OracleProver, ProverError};
use super::types::{OracleProof, PublicInputs};

/// Public inputs for a resolved market from live source responses.
pub fn build_public_inputs(
    responses: &[SourceResponse],
    result: &AggregationResult,
    timestamp: u64,
) -> PublicInputs {
    let included = included_source_ids(responses);
    let included_ids: HashSet<_> =
        included.iter().map(String::as_str).collect();
    let hash = agreement_hash(responses, &included_ids);
    PublicInputs::from_aggregation(result, hash, timestamp)
}

/// Build public inputs and circuit witness, then prove.
pub fn prove_responses(
    prover: &OracleProver,
    responses: &[SourceResponse],
    timestamp: u64,
) -> Result<(OracleProof, PublicInputs), ProverError> {
    let (circuit, result) = build_witness(responses)
        .map_err(|e| ProverError::Witness(e.to_string()))?;
    let public_inputs = build_public_inputs(responses, &result, timestamp);
    let proof = prover.prove(circuit)?;
    Ok((proof, public_inputs))
}
