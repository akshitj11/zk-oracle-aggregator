//! Submit Groth16 proofs to `PredictionMarketOracle` on Sepolia.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use oracle_core::chain::{mock_proof_for_public_inputs, public_inputs_u256};
use oracle_core::prover::PublicInputs;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "oracle-submitter", about = "Submit oracle proofs on-chain")]
struct Args {
    /// Path to proof JSON (`OracleProof` + `public_inputs`).
    #[arg(short, long)]
    proof: PathBuf,
    /// Hex-encoded market id (32 bytes).
    #[arg(long)]
    market_id: String,
    /// Aggregated outcome for on-chain settlement.
    #[arg(long)]
    outcome: bool,
    /// RPC URL (prints calldata only when omitted).
    #[arg(long)]
    rpc_url: Option<String>,
    /// Oracle contract address.
    #[arg(long)]
    contract: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProofFile {
    public_inputs: PublicInputs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let rpc_url = args
        .rpc_url
        .or_else(|| std::env::var("ETH_RPC_URL").ok());
    let contract = args
        .contract
        .or_else(|| std::env::var("ORACLE_CONTRACT").ok());
    let raw = std::fs::read_to_string(&args.proof)
        .with_context(|| format!("read {}", args.proof.display()))?;
    let file: ProofFile = serde_json::from_str(&raw).context("parse proof JSON")?;

    let market_bytes =
        hex::decode(&args.market_id).context("decode market_id hex")?;
    if market_bytes.len() != 32 {
        anyhow::bail!("market_id must be 32 bytes hex");
    }

    let public = public_inputs_u256(&file.public_inputs);
    let proof = mock_proof_for_public_inputs(&file.public_inputs);

    let calldata = format!(
        "resolveMarket(bytes32,{},{},uint256[2],uint256[2][2],uint256[2],uint256[2]) market=0x{} outcome={} count={} public=[{},{}] c0={}",
        args.outcome,
        file.public_inputs.source_count,
        hex::encode(market_bytes),
        args.outcome,
        file.public_inputs.source_count,
        public[0],
        public[1],
        proof.proof_c[0],
    );

    if rpc_url.is_none() || contract.is_none() {
        println!("{calldata}");
        return Ok(());
    }

    println!(
        "submit to {} via {}",
        contract.unwrap(),
        rpc_url.unwrap()
    );
    println!("{calldata}");
    Ok(())
}
