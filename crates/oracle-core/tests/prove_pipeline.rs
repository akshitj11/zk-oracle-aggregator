//! End-to-end prove pipeline over sample responses.

use oracle_core::fetcher::{Outcome, SourceResponse};
use oracle_core::prover::{OracleProver, prove_responses};

fn response(id: &str, outcome: Outcome, confidence: f64) -> SourceResponse {
    SourceResponse {
        source_id: id.to_owned(),
        outcome,
        confidence,
        fetched_at: 1_700_000_000,
        raw_hash: {
            let mut hash = [0u8; 32];
            hash[0] = id.as_bytes()[0];
            hash
        },
    }
}

#[test]
fn prove_responses_round_trip() {
    let prover = OracleProver::generate_keys().expect("setup");
    let responses = vec![
        response("src-a", Outcome::Yes, 0.92),
        response("src-b", Outcome::Yes, 0.88),
        response("src-c", Outcome::No, 0.15),
    ];

    let (proof, public) = prove_responses(&prover, &responses, 1_700_000_100).expect("prove");
    assert!(public.outcome);
    assert_eq!(public.source_count, 3);

    let ok = prover
        .verifier()
        .verify(&proof, &public.to_verifier_inputs())
        .expect("verify");
    assert!(ok);
}
