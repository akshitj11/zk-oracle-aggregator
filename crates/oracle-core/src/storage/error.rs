//! Storage layer errors.

use thiserror::Error;

/// Errors from [`OracleStore`] operations.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(String),
    #[error("proof not found for market")]
    NotFound,
}

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        StoreError::Database(err.to_string())
    }
}
