//! Verify a Groth16 oracle proof from stdin JSON.

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use oracle_core::prover::{OracleProof, OracleVerifier, PublicInputs};
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "oracle-verifier", about = "Verify an oracle Groth16 proof")]
struct Args {
    /// Load verifying key from file instead of JSON payload.
    #[arg(long)]
    verifying_key: Option<PathBuf>,
}

#[derive(Deserialize)]
struct VerifierInput {
    proof: OracleProof,
    public_inputs: PublicInputs,
    verifying_key: Option<Vec<u8>>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut json = String::new();
    io::stdin()
        .read_to_string(&mut json)
        .context("read stdin")?;
    let input: VerifierInput =
        serde_json::from_str(&json).context("parse verifier JSON")?;

    let vk_bytes = if let Some(path) = &args.verifying_key {
        fs::read(path).with_context(|| format!("read {}", path.display()))?
    } else {
        input.verifying_key.ok_or_else(|| {
            anyhow::anyhow!(
                "verifying_key missing; pass --verifying-key or embed in JSON"
            )
        })?
    };

    let verifier = OracleVerifier::from_bytes(&vk_bytes).context("load vk")?;
    let ok = verifier
        .verify(&input.proof, &input.public_inputs.to_verifier_inputs())
        .context("verify")?;

    if ok {
        println!("valid");
        Ok(())
    } else {
        anyhow::bail!("invalid proof");
    }
}
