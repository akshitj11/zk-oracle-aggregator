//! OracleStore round-trip tests (require DATABASE_URL).

use oracle_core::aggregator::AggregationResult;
use oracle_core::fetcher::{Outcome, SourceResponse};
use oracle_core::prover::{OracleProof, PublicInputs};
use oracle_core::storage::OracleStore;
use rust_decimal::Decimal;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn sample_result() -> AggregationResult {
    AggregationResult {
        outcome: Outcome::Yes,
        confidence: Decimal::new(85, 2),
        source_count: 2,
        agreement_ratio: Decimal::new(100, 2),
        disputed: false,
    }
}

fn sample_proof() -> OracleProof {
    OracleProof {
        proof_bytes: vec![1, 2, 3, 4],
        generated_at: 1_700_000_000,
    }
}

fn sample_public_inputs() -> PublicInputs {
    PublicInputs {
        outcome: true,
        source_count: 2,
        agreement_hash: [7u8; 32],
        timestamp: 1_700_000_000,
    }
}

fn sample_response(id: &str) -> SourceResponse {
    SourceResponse {
        source_id: id.to_string(),
        outcome: Outcome::Yes,
        confidence: 0.9,
        fetched_at: 1_700_000_000,
        raw_hash: [id.as_bytes()[0]; 32],
    }
}

#[tokio::test]
async fn save_and_get_proof_round_trip() {
    let url = database_url().expect("DATABASE_URL required for storage tests");
    let store = OracleStore::connect(&url).await.expect("connect");

    let market_id = [42u8; 32];
    let proof_id = store
        .save_proof(
            &market_id,
            &sample_result(),
            &sample_proof(),
            &sample_public_inputs(),
        )
        .await
        .expect("save");

    let loaded = store
        .get_proof(&market_id)
        .await
        .expect("get")
        .expect("row");

    assert_eq!(loaded.id, proof_id);
    assert_eq!(loaded.market_id, market_id);
    assert!(loaded.outcome);
    assert_eq!(loaded.source_count, 2);
    assert_eq!(loaded.proof.proof_bytes, sample_proof().proof_bytes);
    assert_eq!(loaded.public_inputs.source_count, 2);
}

#[tokio::test]
async fn reputation_upsert_updates_weight() {
    let url = database_url().expect("DATABASE_URL required for storage tests");
    let store = OracleStore::connect(&url).await.expect("connect");

    store
        .update_reputation("src-a", true)
        .await
        .expect("update");
    store
        .update_reputation("src-a", false)
        .await
        .expect("update");

    let record = store
        .get_reputation("src-a")
        .await
        .expect("get")
        .expect("row");

    assert_eq!(record.correct_count, 1);
    assert_eq!(record.total_count, 2);
}

#[tokio::test]
async fn save_source_responses_links_to_proof() {
    let url = database_url().expect("DATABASE_URL required for storage tests");
    let store = OracleStore::connect(&url).await.expect("connect");

    let market_id = [99u8; 32];
    let proof_id = store
        .save_proof(
            &market_id,
            &sample_result(),
            &sample_proof(),
            &sample_public_inputs(),
        )
        .await
        .expect("save");

    let responses = vec![sample_response("src-a"), sample_response("src-b")];
    store
        .save_source_responses(proof_id, &responses)
        .await
        .expect("save responses");

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM source_responses WHERE proof_id = $1",
    )
    .bind(proof_id)
    .fetch_one(store.pool())
    .await
    .expect("count");

    assert_eq!(count.0, 2);
}
