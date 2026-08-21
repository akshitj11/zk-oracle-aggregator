//! REST API for oracle proofs and resolution.

pub mod api;
pub mod config;
pub mod middleware;
pub mod router;
pub mod state;

pub use router::build_router;

use std::net::SocketAddr;

use anyhow::Context;
use oracle_core::prover::OracleProver;
use oracle_core::storage::OracleStore;
use reqwest::Client;
use tracing_subscriber::EnvFilter;

use crate::config::{load_source_pairs, ServerConfig};
use crate::state::AppState;

/// Start the HTTP server using environment configuration.
pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    let cfg = ServerConfig::from_env()?;
    let store = OracleStore::connect(&cfg.database_url).await?;
    let source_pairs = load_source_pairs(&cfg.sources_config)?;

    let prover = load_prover(&cfg)?;
    let verifier = prover.verifier();

    let state = AppState::new(
        store,
        prover,
        verifier,
        Client::new(),
        source_pairs,
        cfg.api_key,
    );

    let app = build_router(state);
    let addr: SocketAddr = cfg.listen_addr.parse().context("parse LISTEN_ADDR")?;
    tracing::info!(%addr, "oracle-server listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn load_prover(cfg: &ServerConfig) -> anyhow::Result<OracleProver> {
    match (&cfg.proving_key_path, &cfg.verifying_key_path) {
        (Some(pk), Some(vk)) => {
            let pk_bytes = std::fs::read(pk).with_context(|| format!("read {}", pk.display()))?;
            let vk_bytes =
                std::fs::read(vk).with_context(|| format!("read {}", vk.display()))?;
            OracleProver::from_key_bytes(&pk_bytes, &vk_bytes).context("load keys")
        }
        (None, None) => OracleProver::generate_keys().context("generate keys"),
        _ => anyhow::bail!("set both PROVING_KEY_PATH and VERIFYING_KEY_PATH, or neither"),
    }
}
