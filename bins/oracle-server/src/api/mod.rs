//! API module exports.

mod routes;
mod types;

pub use routes::{
    get_proof, get_reputation, health, metrics, resolve_market,
    verify_stored_proof,
};
pub use types::{ApiError, ApiErrorBody, HealthResponse, ResolveResponse};
