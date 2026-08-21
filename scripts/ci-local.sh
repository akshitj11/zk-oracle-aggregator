#!/usr/bin/env bash
# Local CI mirror of .github/workflows/ci.yaml — run before every commit.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== cargo build =="
cargo build --workspace --all-targets --all-features

echo "== cargo fmt =="
cargo fmt --all --check

echo "== cargo clippy =="
cargo clippy --workspace --all-targets --all-features -- -Dwarnings

echo "== cargo doc =="
cargo doc --workspace --all-features

echo "== cargo test doc =="
cargo test --doc --workspace --all-features

echo "== cargo nextest =="
if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run --workspace --all-targets --all-features --no-tests=pass
else
  cargo test --workspace --all-targets --all-features
fi

echo "== typos =="
if command -v typos >/dev/null 2>&1; then
  typos
else
  echo "skip: typos not installed"
fi

echo "== taplo =="
if command -v taplo >/dev/null 2>&1; then
  taplo fmt --check --diff
else
  echo "skip: taplo not installed"
fi

echo "== markdownlint =="
if command -v markdownlint-cli2 >/dev/null 2>&1; then
  markdownlint-cli2 README.md "docs/**/*.md"
else
  echo "skip: markdownlint-cli2 not installed"
fi

echo "== yamlfmt =="
if command -v yamlfmt >/dev/null 2>&1; then
  yamlfmt --lint .
else
  echo "skip: yamlfmt not installed"
fi

echo "== cargo deny =="
if command -v cargo-deny >/dev/null 2>&1; then
  cargo deny check
else
  echo "skip: cargo-deny not installed"
fi

echo "== cargo audit =="
if command -v cargo-audit >/dev/null 2>&1; then
  cargo audit
else
  echo "skip: cargo-audit not installed"
fi

echo "== secrets grep =="
if git ls-files '*.pk' 'keys/pk.bin' 'keys/*.pk' 'vk.bin' 'proof.bin' 2>/dev/null | grep -q .; then
  echo "Committed proving key or proof artifact detected"
  exit 1
fi

echo "== MSRV check =="
if command -v cargo-msrv >/dev/null 2>&1; then
  MSRV="$(cargo msrv --manifest-path crates/oracle-core/Cargo.toml show --output-format minimal | tr -d '[:space:]')"
  rustup run "$MSRV" cargo check -p oracle-core --all-targets --locked
  rustup run "$MSRV" cargo test -p oracle-core --locked
else
  echo "skip: cargo-msrv not installed"
fi

echo "ci-local: OK"
