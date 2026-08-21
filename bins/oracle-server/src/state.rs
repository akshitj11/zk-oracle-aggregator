//! Shared application state.

use std::sync::Arc;

use oracle_core::prover::{OracleProver, OracleVerifier};
use oracle_core::storage::OracleStore;
use reqwest::Client;

/// State shared across API handlers.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub store: OracleStore,
    pub prover: OracleProver,
    pub verifier: OracleVerifier,
    pub client: Client,
    pub source_pairs: Vec<(String, String)>,
    pub api_key: Option<String>,
}

impl AppState {
    pub fn new(
        store: OracleStore,
        prover: OracleProver,
        verifier: OracleVerifier,
        client: Client,
        source_pairs: Vec<(String, String)>,
        api_key: Option<String>,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                store,
                prover,
                verifier,
                client,
                source_pairs,
                api_key,
            }),
        }
    }

    pub fn store(&self) -> &OracleStore {
        &self.inner.store
    }

    pub fn prover(&self) -> &OracleProver {
        &self.inner.prover
    }

    pub fn verifier(&self) -> &OracleVerifier {
        &self.inner.verifier
    }

    pub fn client(&self) -> &Client {
        &self.inner.client
    }

    pub fn source_pairs(&self) -> &[(String, String)] {
        &self.inner.source_pairs
    }

    pub fn api_key(&self) -> Option<&str> {
        self.inner.api_key.as_deref()
    }
}
