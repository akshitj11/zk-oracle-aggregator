//! Integration test using the three-source wiremock fixture.

mod common;

use oracle_core::{aggregate, fetch_all_sources, Outcome};
use reqwest::Client;

#[tokio::test]
async fn fetch_three_mock_sources_aggregate_yes() {
    let cluster =
        common::mock_sources::MockSourceCluster::three_source_consensus().await;
    let client = Client::new();
    let responses = fetch_all_sources(&client, &cluster.sources).await;

    assert_eq!(responses.len(), 3);
    let result = aggregate(&responses);
    assert_eq!(result.outcome, Outcome::Yes);
    assert!(!result.disputed);
    assert_eq!(result.source_count, 3);
}
