//! Core types and modules for the ZK oracle aggregator.

#![cfg_attr(not(test), warn(clippy::unwrap_used, clippy::expect_used))]

pub mod aggregator;
pub mod chain;
pub mod circuit;
pub mod fetcher;
pub mod prover;
pub mod storage;

/// Maximum source slots in the ZK circuit and fetch pipeline.
pub const MAX_SOURCES: usize = 16;

pub use aggregator::{
    aggregate, remove_outliers, weighted_median, AggregationResult,
};
pub use circuit::OracleCircuit;
pub use fetcher::{
    fetch_all_sources, fetch_all_sources_with_limit, fetch_source,
    parse_response, Outcome, ParseError, SourceResponse,
};
pub use prover::{
    build_public_inputs, prove_responses, OracleProof, OracleProver,
    OracleVerifier, ProverError, PublicInputs,
};
pub use storage::{OracleStore, ReputationRecord, StoreError, StoredProof};
