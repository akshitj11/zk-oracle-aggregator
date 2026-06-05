//! Core types and modules for the ZK oracle aggregator.

pub mod aggregator;
pub mod fetcher;

pub use aggregator::{
    aggregate, remove_outliers, weighted_median, AggregationResult,
};
pub use fetcher::{fetch_all_sources, fetch_source, Outcome, SourceResponse};
