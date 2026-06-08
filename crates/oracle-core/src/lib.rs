//! Core types and modules for the ZK oracle aggregator.

#![cfg_attr(not(test), warn(clippy::unwrap_used, clippy::expect_used))]

pub mod aggregator;
pub mod fetcher;

pub use aggregator::{
    aggregate, remove_outliers, weighted_median, AggregationResult,
};
pub use fetcher::{
    fetch_all_sources, fetch_source, parse_response, Outcome, ParseError,
    SourceResponse,
};
