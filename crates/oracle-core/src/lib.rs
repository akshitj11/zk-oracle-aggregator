//! Core types and modules for the ZK oracle aggregator.

pub mod fetcher;

pub use fetcher::{fetch_all_sources, fetch_source, Outcome, SourceResponse};
