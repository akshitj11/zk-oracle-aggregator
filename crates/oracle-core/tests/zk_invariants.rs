//! ZK invariant tests (Z1–Z8).

use std::collections::HashSet;

use ark_bn254::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};

use oracle_core::aggregator::aggregate;
use oracle_core::circuit::{
    agreement_hash, build_witness, WitnessError, MAJORITY_MARGIN_BITS,
};
use oracle_core::fetcher::{Outcome, SourceResponse};
use oracle_core::prover::{OracleProver, PublicInputs};
use oracle_core::MAX_SOURCES;

fn response(id: &str, outcome: Outcome, confidence: f64) -> SourceResponse {
    SourceResponse {
        source_id: id.to_owned(),
        outcome,
        confidence,
        fetched_at: 1,
        raw_hash: {
            let mut hash = [0u8; 32];
            hash[0] = id.as_bytes()[0];
            hash
        },
    }
}

#[test]
fn z6_witness_matches_aggregate() {
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::Yes, 0.8),
        response("c", Outcome::No, 0.1),
    ];
    let expected = aggregate(&responses);
    let (_, result) = build_witness(&responses).unwrap();
    assert_eq!(result.outcome, expected.outcome);
    assert_eq!(result.source_count, expected.source_count);
    assert_eq!(result.disputed, expected.disputed);
}

#[test]
fn z6_rejects_disputed_witness() {
    assert!(matches!(build_witness(&[]), Err(WitnessError::Disputed)));
}

#[test]
fn z3_source_count_matches_inclusion_flags() {
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::Yes, 0.8),
    ];
    let (circuit, _) = build_witness(&responses).unwrap();
    let included: u64 = circuit
        .source_included
        .iter()
        .take(MAX_SOURCES)
        .filter(|v| **v == Some(Fr::from(1u64)))
        .count() as u64;
    assert_eq!(circuit.source_count, Some(Fr::from(included)));
}

#[test]
fn z1_outcome_witnesses_are_binary() {
    let responses = vec![response("a", Outcome::Yes, 0.9)];
    let (circuit, _) = build_witness(&responses).unwrap();
    for outcome in circuit.source_outcomes.iter().take(1) {
        let v = outcome.unwrap();
        assert!(v == Fr::from(0u64) || v == Fr::from(1u64));
    }
}

#[test]
fn z2_excluded_sources_do_not_count_toward_total() {
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::Yes, 0.9),
        response("c", Outcome::Yes, 0.9),
        response("d", Outcome::No, 0.9),
    ];
    let (circuit, result) = build_witness(&responses).unwrap();
    assert_eq!(result.outcome, Outcome::Yes);
    assert_eq!(result.source_count, 3);
    let d_index = 3;
    assert_eq!(circuit.source_included[d_index], Some(Fr::from(0u64)));
}

#[test]
fn included_source_ids_matches_outlier_filter() {
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::Yes, 0.9),
        response("c", Outcome::Yes, 0.9),
        response("d", Outcome::No, 0.9),
    ];
    let ids = oracle_core::circuit::included_source_ids(&responses);
    assert!(ids.contains("a"));
    assert!(!ids.contains("d"));
}

#[test]
fn z5_agreement_hash_is_stable() {
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::No, 0.2),
    ];
    let ids: HashSet<_> = ["a", "b"].into_iter().collect();
    let h1 = agreement_hash(&responses, &ids);
    let h2 = agreement_hash(&responses, &ids);
    assert_eq!(h1, h2);
}

#[test]
fn z4_circuit_satisfied_for_witness() {
    let responses = vec![
        response("a", Outcome::Yes, 0.95),
        response("b", Outcome::Yes, 0.90),
        response("c", Outcome::No, 0.05),
    ];
    let (circuit, _) = build_witness(&responses).unwrap();
    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit.generate_constraints(cs.clone()).unwrap();
    assert!(cs.is_satisfied().unwrap());
}

#[test]
fn z7_z8_groth16_verify_invariants() {
    let prover = OracleProver::generate_keys().unwrap();
    let responses = vec![response("a", Outcome::Yes, 0.9)];
    let (circuit, result) = build_witness(&responses).unwrap();
    let ids: HashSet<_> = ["a"].into_iter().collect();
    let public = PublicInputs::from_aggregation(
        &result,
        agreement_hash(&responses, &ids),
        1,
    );
    let proof = prover.prove(circuit).unwrap();
    assert!(prover
        .verifier()
        .verify(&proof, &public.to_verifier_inputs())
        .unwrap());
}

#[test]
fn witness_pads_to_max_sources() {
    let responses = vec![response("a", Outcome::Yes, 0.9)];
    let (circuit, _) = build_witness(&responses).unwrap();
    assert_eq!(circuit.source_outcomes.len(), MAX_SOURCES);
    assert_eq!(circuit.majority_margin_bits.len(), MAJORITY_MARGIN_BITS);
}
