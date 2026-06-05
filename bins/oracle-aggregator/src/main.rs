use std::io::{self, Read};

use anyhow::Context;
use oracle_core::{aggregate, SourceResponse};

fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("read stdin")?;

    let responses: Vec<SourceResponse> =
        serde_json::from_str(&input).context("parse SourceResponse[] json")?;

    let result = aggregate(&responses);
    let json = serde_json::to_string_pretty(&result)?;
    println!("{json}");
    Ok(())
}
