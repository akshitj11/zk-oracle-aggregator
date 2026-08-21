//! Field encoding helpers for circuit witnesses.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};

use crate::fetcher::Outcome;

/// Scale confidence in `[0,1]` to an integer weight for in-circuit arithmetic.
pub const CONFIDENCE_SCALE: u64 = 1_000_000;

/// Map YES/NO to 1/0; Unknown maps to 0 weight via outcome encoding.
pub fn outcome_to_field(outcome: Outcome) -> Fr {
    match outcome {
        Outcome::Yes => Fr::from(1u64),
        Outcome::No | Outcome::Unknown => Fr::from(0u64),
    }
}

/// Encode confidence as a fixed-point field element.
pub fn confidence_to_field(confidence: f64) -> Fr {
    let scaled =
        (confidence.clamp(0.0, 1.0) * CONFIDENCE_SCALE as f64).round() as u64;
    Fr::from(scaled)
}

/// Embed the first 8 bytes of a hash into the field (fits BN254 scalar).
#[allow(dead_code)]
pub fn hash_prefix_to_field(hash: &[u8; 32]) -> Fr {
    let mut limb = [0u8; 8];
    limb.copy_from_slice(&hash[..8]);
    Fr::from(u64::from_le_bytes(limb))
}

/// Convert public outcome bool to a field element.
pub fn bool_to_field(value: bool) -> Fr {
    Fr::from(u64::from(value))
}

/// Serialize field elements for Groth16 verify (uncompressed big-endian limbs).
#[allow(dead_code)]
pub fn field_to_bytes(value: &Fr) -> [u8; 32] {
    let bigint = value.into_bigint();
    let mut bytes = [0u8; 32];
    let limb_bytes = bigint.to_bytes_le();
    let len = limb_bytes.len().min(32);
    bytes[..len].copy_from_slice(&limb_bytes[..len]);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_yes_is_one() {
        assert_eq!(outcome_to_field(Outcome::Yes), Fr::from(1u64));
    }

    #[test]
    fn confidence_scales_to_field() {
        let f = confidence_to_field(0.5);
        assert_eq!(f, Fr::from(CONFIDENCE_SCALE / 2));
    }
}
