//! Postgres-backed proof archive and source reputation.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use super::error::StoreError;

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
}
