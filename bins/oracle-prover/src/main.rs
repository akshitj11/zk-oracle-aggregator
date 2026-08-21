//! Generate a Groth16 proof from aggregated source responses on stdin.

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use oracle_core::fetcher::SourceResponse;
use oracle_core::prover::{prove_responses, OracleProver};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "oracle-prover", about = "Prove honest oracle aggregation")]
struct Args {
    /// Unix timestamp (seconds) recorded in public inputs.
    #[arg(long, default_value_t = 0)]
    timestamp: u64,
    /// Load proving key from a file (uncompressed bytes).
    #[arg(long)]
    proving_key: Option<PathBuf>,
    /// Load verifying key from a file (uncompressed bytes).
    #[arg(long)]
    verifying_key: Option<PathBuf>,
    /// Write freshly generated proving key bytes to this path.
    #[arg(long)]
    write_proving_key: Option<PathBuf>,
    /// Write freshly generated verifying key bytes to this path.
    #[arg(long)]
    write_verifying_key: Option<PathBuf>,
}

#[derive(Serialize)]
struct ProverOutput {
    proof: oracle_core::prover::OracleProof,
    public_inputs: oracle_core::prover::PublicInputs,
    verifying_key: Vec<u8>,
}

fn load_or_generate_keys(args: &Args) -> Result<OracleProver> {
    match (&args.proving_key, &args.verifying_key) {
        (Some(pk_path), Some(vk_path)) => {
            let pk = fs::read(pk_path)
                .with_context(|| format!("read {}", pk_path.display()))?;
            let vk = fs::read(vk_path)
                .with_context(|| format!("read {}", vk_path.display()))?;
            OracleProver::from_key_bytes(&pk, &vk).context("load keys")
        }
        (None, None) => OracleProver::generate_keys().context("trusted setup"),
        _ => anyhow::bail!(
            "pass both --proving-key and --verifying-key, or neither"
        ),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut json = String::new();
    io::stdin()
        .read_to_string(&mut json)
        .context("read stdin")?;
    let responses: Vec<SourceResponse> =
        serde_json::from_str(&json).context("parse source responses JSON")?;

    let prover = load_or_generate_keys(&args)?;

    if let Some(path) = &args.write_proving_key {
        fs::write(path, prover.proving_key_bytes().context("serialize pk")?)
            .with_context(|| format!("write {}", path.display()))?;
    }
    if let Some(path) = &args.write_verifying_key {
        fs::write(path, prover.verifying_key_bytes().context("serialize vk")?)
            .with_context(|| format!("write {}", path.display()))?;
    }

    let (proof, public_inputs) =
        prove_responses(&prover, &responses, args.timestamp)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    let verifying_key = prover.verifying_key_bytes().context("serialize vk")?;

    let output = ProverOutput {
        proof,
        public_inputs,
        verifying_key,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
