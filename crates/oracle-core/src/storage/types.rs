//! Stored proof and reputation records.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fetcher::Outcome;
use crate::prover::{OracleProof, PublicInputs};

/// Proof row loaded from Postgres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredProof {
    pub id: Uuid,
    pub market_id: Vec<u8>,
    pub outcome: bool,
    pub source_count: i32,
    pub agreement_ratio: Decimal,
    pub confidence: Decimal,
    pub proof: OracleProof,
    pub public_inputs: PublicInputs,
    pub onchain_tx_hash: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

/// Source reputation snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReputationRecord {
    pub source_id: String,
    pub correct_count: i32,
    pub total_count: i32,
    pub current_weight: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Outcome label for `source_responses.outcome` column.
pub fn outcome_to_db(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Yes => "YES",
        Outcome::No => "NO",
        Outcome::Unknown => "UNKNOWN",
    }
}
