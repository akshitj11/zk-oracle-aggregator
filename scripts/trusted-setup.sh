#!/usr/bin/env bash
# Powers of Tau + circuit-specific Groth16 setup for production keys.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KEYS_DIR="${ROOT}/keys"
PTAU="${KEYS_DIR}/powers_of_tau.ptau"

mkdir -p "$KEYS_DIR"

echo "Download Powers of Tau (adjust URL for your ceremony):"
echo "  curl -L -o '$PTAU' https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_16.ptau"

echo "Generate dev keys locally (NOT for production):"
echo "  cargo run -p oracle-prover -- --write-proving-key '$KEYS_DIR/pk.bin' --write-verifying-key '$KEYS_DIR/vk.bin' < fixtures/sample-responses.json"

echo "Production: run a multi-party ceremony and store pk offline; commit only vk hash at API boundary."
