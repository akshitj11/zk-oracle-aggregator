//! Proving key serialization round-trip.

use oracle_core::prover::OracleProver;

#[test]
fn proving_and_verifying_keys_round_trip_bytes() {
    let prover = OracleProver::generate_keys().expect("setup");
    let pk = prover.proving_key_bytes().expect("pk bytes");
    let vk = prover.verifying_key_bytes().expect("vk bytes");

    let loaded = OracleProver::from_key_bytes(&pk, &vk).expect("load keys");
    assert_eq!(pk, loaded.proving_key_bytes().expect("pk round trip"));
    assert_eq!(vk, loaded.verifying_key_bytes().expect("vk round trip"));
}
