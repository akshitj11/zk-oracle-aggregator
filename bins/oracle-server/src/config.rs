//! Shared server configuration from environment.

use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;

/// Runtime configuration for `oracle-server`.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub database_url: String,
    pub sources_config: PathBuf,
    pub proving_key_path: Option<PathBuf>,
    pub verifying_key_path: Option<PathBuf>,
    pub api_key: Option<String>,
    pub listen_addr: String,
}

impl ServerConfig {
    /// Load from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL")?;
        let sources_config = std::env::var("SOURCES_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from("config/sources.integration.toml")
            });
        let proving_key_path =
            std::env::var("PROVING_KEY_PATH").ok().map(PathBuf::from);
        let verifying_key_path =
            std::env::var("VERIFYING_KEY_PATH").ok().map(PathBuf::from);
        let api_key = std::env::var("ORACLE_API_KEY").ok();
        let listen_addr = std::env::var("LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        Ok(Self {
            database_url,
            sources_config,
            proving_key_path,
            verifying_key_path,
            api_key,
            listen_addr,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct SourcesFile {
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub url: String,
}

pub fn load_source_pairs(
    path: &std::path::Path,
) -> anyhow::Result<Vec<(String, String)>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read sources config {}", path.display()))?;
    let file: SourcesFile =
        toml::from_str(&raw).context("parse sources config")?;
    Ok(file.sources.into_iter().map(|s| (s.id, s.url)).collect())
}
