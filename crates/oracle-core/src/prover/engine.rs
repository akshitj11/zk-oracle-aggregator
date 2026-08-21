//! Groth16 setup, prove, and verify for [`OracleCircuit`](crate::circuit::OracleCircuit).

use std::time::{SystemTime, UNIX_EPOCH};

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use rand::rngs::OsRng;
use thiserror::Error;

use crate::circuit::{padded_empty_circuit, OracleCircuit};

use super::types::OracleProof;

/// Errors from the Groth16 prover or verifier.
#[derive(Debug, Error)]
pub enum ProverError {
    #[error("groth16 setup failed: {0}")]
    Setup(String),
    #[error("groth16 prove failed: {0}")]
    Prove(String),
    #[error("groth16 verify failed: {0}")]
    Verify(String),
    #[error("proof serialization failed: {0}")]
    Serialize(String),
    #[error("proof deserialization failed: {0}")]
    Deserialize(String),
}

/// Loaded verifying key for proof checks.
pub struct OracleVerifier {
    verifying_key: VerifyingKey<Bn254>,
}

impl OracleVerifier {
    /// Load a verifying key from uncompressed bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProverError> {
        let verifying_key = VerifyingKey::deserialize_uncompressed(bytes)
            .map_err(|e| ProverError::Deserialize(e.to_string()))?;
        Ok(Self { verifying_key })
    }

    /// Verify a proof against the circuit public inputs.
    pub fn verify(&self, proof: &OracleProof, public_inputs: &[Fr]) -> Result<bool, ProverError> {
        let proof = Proof::deserialize_uncompressed(proof.proof_bytes.as_slice())
            .map_err(|e| ProverError::Deserialize(e.to_string()))?;

        Groth16::<Bn254>::verify(&self.verifying_key, public_inputs, &proof)
            .map_err(|e| ProverError::Verify(e.to_string()))
    }
}

/// Loaded proving and verifying keys for oracle proofs.
pub struct OracleProver {
    proving_key: ProvingKey<Bn254>,
    verifying_key: VerifyingKey<Bn254>,
}

impl OracleProver {
    /// Run a local trusted setup for development and tests.
    pub fn generate_keys() -> Result<Self, ProverError> {
        let circuit = padded_empty_circuit();
        let mut rng = OsRng;
        let (proving_key, verifying_key) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)
            .map_err(|e| ProverError::Setup(e.to_string()))?;
        Ok(Self {
            proving_key,
            verifying_key,
        })
    }

    /// Borrow a verifier view of the proving keys.
    pub fn verifier(&self) -> OracleVerifier {
        OracleVerifier {
            verifying_key: self.verifying_key.clone(),
        }
    }

    /// Serialize the verifying key for distribution to verifiers.
    pub fn verifying_key_bytes(&self) -> Result<Vec<u8>, ProverError> {
        let mut bytes = Vec::new();
        self.verifying_key
            .serialize_uncompressed(&mut bytes)
            .map_err(|e| ProverError::Serialize(e.to_string()))?;
        Ok(bytes)
    }

    /// Produce a Groth16 proof for the given witness circuit.
    pub fn prove(&self, circuit: OracleCircuit) -> Result<OracleProof, ProverError> {
        let mut rng = OsRng;
        let proof = Groth16::<Bn254>::prove(&self.proving_key, circuit, &mut rng)
            .map_err(|e| ProverError::Prove(e.to_string()))?;

        let mut proof_bytes = Vec::new();
        proof
            .serialize_uncompressed(&mut proof_bytes)
            .map_err(|e| ProverError::Serialize(e.to_string()))?;

        let generated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(OracleProof {
            proof_bytes,
            generated_at,
        })
    }
}
