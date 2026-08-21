#!/usr/bin/env bash
# Smoke test: prove and verify fixture responses without network.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE="${ROOT}/fixtures/sample-responses.json"
PK="${TMPDIR:-/tmp}/oracle-pk.bin"
VK="${TMPDIR:-/tmp}/oracle-vk.bin"
PROOF="${TMPDIR:-/tmp}/oracle-proof.json"

cd "$ROOT"

cargo build -q -p oracle-prover -p oracle-verifier

cargo run -q -p oracle-prover -- \
  --write-proving-key "$PK" \
  --write-verifying-key "$VK" \
  --timestamp 1700000100 \
  < "$FIXTURE" > "$PROOF"

cargo run -q -p oracle-verifier -- --verifying-key "$VK" < "$PROOF"

echo "prove-verify smoke ok"
