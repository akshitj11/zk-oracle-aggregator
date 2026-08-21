//! PostgreSQL storage for proofs and source reputation.

mod error;
mod store;
mod types;

pub use error::StoreError;
pub use store::OracleStore;
pub use types::{outcome_to_db, ReputationRecord, StoredProof};
