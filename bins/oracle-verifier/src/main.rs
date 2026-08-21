//! Verify a Groth16 oracle proof from stdin JSON.

use std::io::{self, Read};

use anyhow::{Context, Result};
use clap::Parser;
use oracle_core::prover::{OracleProof, OracleVerifier, PublicInputs};
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "oracle-verifier", about = "Verify an oracle Groth16 proof")]
struct Args {}

#[derive(Deserialize)]
struct VerifierInput {
    proof: OracleProof,
    public_inputs: PublicInputs,
    verifying_key: Vec<u8>,
}

fn main() -> Result<()> {
    let _args = Args::parse();

    let mut json = String::new();
    io::stdin()
        .read_to_string(&mut json)
        .context("read stdin")?;
    let input: VerifierInput = serde_json::from_str(&json).context("parse verifier JSON")?;

    let verifier = OracleVerifier::from_bytes(&input.verifying_key).context("load vk")?;
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
