//! Prove pipeline persisted through OracleStore.

use oracle_core::fetcher::{Outcome, SourceResponse};
use oracle_core::prover::{prove_responses, OracleProver};
use oracle_core::storage::OracleStore;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn response(id: &str, outcome: Outcome, confidence: f64) -> SourceResponse {
    SourceResponse {
        source_id: id.to_string(),
        outcome,
        confidence,
        fetched_at: 1_700_000_100,
        raw_hash: [id.as_bytes()[0]; 32],
    }
}

#[tokio::test]
async fn prove_then_save_and_reload() {
    let url = database_url().expect("DATABASE_URL required");
    let store = OracleStore::connect(&url).await.expect("connect");
    let prover = OracleProver::generate_keys().expect("keys");

    let responses = vec![
        response("src-a", Outcome::Yes, 0.92),
        response("src-b", Outcome::Yes, 0.88),
    ];
    let result = oracle_core::aggregate(&responses);
    let (proof, public_inputs) = prove_responses(&prover, &responses, 1_700_000_100)
        .expect("prove");

    let market_id = [11u8; 32];
    let proof_id = store
        .save_proof(&market_id, &result, &proof, &public_inputs)
        .await
        .expect("save");

    store
        .save_source_responses(proof_id, &responses)
        .await
        .expect("responses");

    let loaded = store
        .get_proof(&market_id)
        .await
        .expect("get")
        .expect("row");

    assert_eq!(loaded.public_inputs.source_count, public_inputs.source_count);
    assert_eq!(loaded.proof.proof_bytes, proof.proof_bytes);
    assert!(loaded.public_inputs.outcome);
}
