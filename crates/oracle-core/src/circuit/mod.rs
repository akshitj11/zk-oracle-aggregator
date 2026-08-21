//! Groth16 oracle circuit (BN254).

mod field;
mod oracle_circuit;
mod witness;

pub use field::{confidence_to_field, outcome_to_field, CONFIDENCE_SCALE};
pub use oracle_circuit::OracleCircuit;
pub use witness::{build_witness, padded_empty_circuit, WitnessError};
