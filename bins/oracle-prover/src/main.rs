//! Generate a Groth16 proof from aggregated source responses on stdin.

use std::collections::HashSet;
use std::io::{self, Read};

use anyhow::{Context, Result};
use clap::Parser;
use oracle_core::circuit::{agreement_hash, build_witness};
use oracle_core::fetcher::SourceResponse;
use oracle_core::prover::{OracleProver, PublicInputs};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "oracle-prover", about = "Prove honest oracle aggregation")]
struct Args {
    /// Unix timestamp (seconds) recorded in public inputs.
    #[arg(long, default_value_t = 0)]
    timestamp: u64,
}

#[derive(Serialize)]
struct ProverOutput {
    proof: oracle_core::prover::OracleProof,
    public_inputs: PublicInputs,
    verifying_key: Vec<u8>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut json = String::new();
    io::stdin()
        .read_to_string(&mut json)
        .context("read stdin")?;
    let responses: Vec<SourceResponse> =
        serde_json::from_str(&json).context("parse source responses JSON")?;

    let (circuit, result) = build_witness(&responses).map_err(|e| anyhow::anyhow!("{e}"))?;

    let filtered = oracle_core::aggregator::remove_outliers(&responses, 0.70);
    let included_ids: HashSet<_> = filtered
        .iter()
        .map(|r| r.source_id.as_str())
        .collect();
    let hash = agreement_hash(&responses, &included_ids);
    let public_inputs =
        PublicInputs::from_aggregation(&result, hash, args.timestamp);

    let prover = OracleProver::generate_keys().context("trusted setup")?;
    let proof = prover.prove(circuit).context("prove")?;
    let verifying_key = prover.verifying_key_bytes().context("serialize vk")?;

    let output = ProverOutput {
        proof,
        public_inputs,
        verifying_key,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
