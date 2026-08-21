//! Groth16 oracle circuit (BN254).

mod field;
mod oracle_circuit;
mod witness;

pub use field::{bool_to_field, confidence_to_field, outcome_to_field, CONFIDENCE_SCALE};
pub use oracle_circuit::{OracleCircuit, MAJORITY_MARGIN_BITS};
pub use witness::{
    agreement_hash, build_witness, included_source_ids, padded_empty_circuit, WitnessError,
};
