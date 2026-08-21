use std::io::{self, Read};

use anyhow::{bail, Context};
use clap::Parser;
use oracle_core::{aggregate, SourceResponse};

const DEFAULT_MAX_STDIN_BYTES: usize = 1_048_576;

#[derive(Parser)]
#[command(name = "oracle-aggregator", about = "Aggregate source responses from stdin JSON")]
struct Args {
    /// Maximum stdin size in bytes.
    #[arg(long, default_value_t = DEFAULT_MAX_STDIN_BYTES)]
    max_stdin_bytes: usize,
}

fn read_stdin_limited(max_bytes: usize) -> anyhow::Result<String> {
    let mut handle = io::stdin().lock();
    let mut buf = Vec::new();
    handle
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut buf)
        .context("read stdin")?;

    if buf.len() > max_bytes {
        bail!("stdin exceeds {max_bytes} byte limit");
    }

    String::from_utf8(buf).context("stdin is not valid utf-8")
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input = read_stdin_limited(args.max_stdin_bytes)?;

    let responses: Vec<SourceResponse> =
        serde_json::from_str(&input).context("parse SourceResponse[] json")?;

    let result = aggregate(&responses);
    let json = serde_json::to_string_pretty(&result)?;
    println!("{json}");
    Ok(())
}
