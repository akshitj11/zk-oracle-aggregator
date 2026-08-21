//! Postgres-backed proof archive and source reputation.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::aggregator::AggregationResult;
use crate::fetcher::SourceResponse;
use crate::prover::{OracleProof, PublicInputs};

use super::error::StoreError;
use super::types::{outcome_to_db, ReputationRecord, StoredProof};

/// Parameterized sqlx access to oracle proofs and reputation tables.
#[derive(Debug, Clone)]
pub struct OracleStore {
    pool: PgPool,
}

impl OracleStore {
    /// Connect to Postgres and verify connectivity.
    pub async fn connect(database_url: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    /// Pool for integration tests and advanced callers.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Persist a Groth16 proof and public inputs for a market.
    pub async fn save_proof(
        &self,
        market_id: &[u8],
        result: &AggregationResult,
        proof: &OracleProof,
        public_inputs: &PublicInputs,
    ) -> Result<Uuid, StoreError> {
        let outcome = result.outcome == crate::fetcher::Outcome::Yes;
        let public_json = serde_json::to_value(public_inputs)
            .map_err(|e| StoreError::Database(e.to_string()))?;

        let row = sqlx::query!(
            r#"
            INSERT INTO oracle_proofs (
                market_id, outcome, source_count, agreement_ratio, confidence,
                proof_bytes, public_inputs
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            market_id,
            outcome,
            result.source_count as i32,
            result.agreement_ratio,
            result.confidence,
            proof.proof_bytes.as_slice(),
            public_json
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// Load the stored proof for a market id byte slice.
    pub async fn get_proof(&self, market_id: &[u8]) -> Result<Option<StoredProof>, StoreError> {
        let row = sqlx::query!(
            r#"
            SELECT
                id,
                market_id,
                outcome,
                source_count,
                agreement_ratio,
                confidence,
                proof_bytes,
                public_inputs,
                onchain_tx_hash,
                created_at
            FROM oracle_proofs
            WHERE market_id = $1
            "#,
            market_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let public_inputs: PublicInputs = serde_json::from_value(row.public_inputs)
            .map_err(|e| StoreError::Database(e.to_string()))?;

        Ok(Some(StoredProof {
            id: row.id,
            market_id: row.market_id,
            outcome: row.outcome,
            source_count: row.source_count,
            agreement_ratio: row.agreement_ratio,
            confidence: row.confidence,
            proof: OracleProof {
                proof_bytes: row.proof_bytes,
                generated_at: row
                    .created_at
                    .unwrap_or_else(Utc::now)
                    .timestamp() as u64,
            },
            public_inputs,
            onchain_tx_hash: row.onchain_tx_hash,
            created_at: row.created_at.unwrap_or_else(Utc::now),
        }))
    }

    /// Batch insert source responses linked to a stored proof.
    pub async fn save_source_responses(
        &self,
        proof_id: Uuid,
        responses: &[SourceResponse],
    ) -> Result<(), StoreError> {
        for response in responses {
            let outcome = outcome_to_db(response.outcome);
            let confidence = Decimal::from_f64_retain(response.confidence.clamp(0.0, 1.0))
                .unwrap_or_default();
            let fetched_at = DateTime::from_timestamp(response.fetched_at as i64, 0)
                .unwrap_or_else(Utc::now);

            sqlx::query!(
                r#"
                INSERT INTO source_responses (
                    proof_id, source_id, outcome, confidence, raw_hash, fetched_at
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                proof_id,
                response.source_id,
                outcome,
                confidence,
                response.raw_hash.as_slice(),
                fetched_at
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Increment reputation counters for a source after resolution.
    pub async fn update_reputation(
        &self,
        source_id: &str,
        was_correct: bool,
    ) -> Result<(), StoreError> {
        let correct_delta = if was_correct { 1 } else { 0 };

        sqlx::query!(
            r#"
            INSERT INTO source_reputation (source_id, correct_count, total_count, current_weight)
            VALUES ($1, $2, 1, 0.5)
            ON CONFLICT (source_id) DO UPDATE SET
                correct_count = source_reputation.correct_count + $2,
                total_count = source_reputation.total_count + 1,
                current_weight = LEAST(
                    1.0,
                    (source_reputation.correct_count + $2)::numeric
                        / (source_reputation.total_count + 1)
                ),
                last_updated = NOW()
            "#,
            source_id,
            correct_delta
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load reputation snapshot for a source id.
    pub async fn get_reputation(
        &self,
        source_id: &str,
    ) -> Result<Option<ReputationRecord>, StoreError> {
        let row = sqlx::query!(
            r#"
            SELECT source_id, correct_count, total_count, current_weight, last_updated
            FROM source_reputation
            WHERE source_id = $1
            "#,
            source_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| ReputationRecord {
            source_id: row.source_id,
            correct_count: row.correct_count.unwrap_or(0),
            total_count: row.total_count.unwrap_or(0),
            current_weight: row.current_weight.unwrap_or_default(),
            last_updated: row.last_updated.unwrap_or_else(Utc::now),
        }))
    }
}
