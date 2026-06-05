use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use oracle_core::fetch_all_sources;
use reqwest::Client;
use serde::Deserialize;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "oracle-fetcher",
    about = "Fetch all configured oracle sources"
)]
struct Args {
    /// Path to sources TOML config.
    #[arg(short, long, default_value = "config/sources.example.toml")]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct SourcesFile {
    sources: Vec<SourceEntry>,
}

#[derive(Debug, Deserialize)]
struct SourceEntry {
    id: String,
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    let args = Args::parse();
    let raw = std::fs::read_to_string(&args.config)
        .with_context(|| format!("read config {}", args.config.display()))?;
    let file: SourcesFile =
        toml::from_str(&raw).context("parse sources config")?;

    let pairs: Vec<(String, String)> =
        file.sources.into_iter().map(|s| (s.id, s.url)).collect();

    let client = Client::new();
    let responses = fetch_all_sources(&client, &pairs).await;

    let json = serde_json::to_string_pretty(&responses)?;
    println!("{json}");
    Ok(())
}
