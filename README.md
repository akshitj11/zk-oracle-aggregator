d# ZK Oracle Aggregator

Manipulation-resistant oracle pipeline for prediction markets: **multi-source fetch**, honest aggregation, and **Groth16 proofs** (BN254) for verifiable resolution.

[![CI](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml/badge.svg)](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml)

## Before vs after

```mermaid
flowchart TB
  subgraph before [Traditional oracle]
    M1[Market] --> V1[Vote / committee] --> C1[Contract trusts result]
  end
  subgraph after [This project]
    M2[Market] --> S2[Many sources] --> A2[Aggregate] --> P2[ZK proof] --> V2[Verify] --> C2[Settle]
  end
```

| | Traditional | This project |
| --- | --- | --- |
| Trust | Governance | Cryptography + consensus |
| Audit | Opaque | Proofs + hashed sources |

Details: [docs/WHY.md](docs/WHY.md) · Architecture: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · Security: [docs/security/invariants.md](docs/security/invariants.md) · [SECURITY.md](SECURITY.md)

## Quick start

**Requirements:** Rust 1.88+, Docker (for Postgres).

```bash
# Build and test
cargo build --workspace
cargo test --workspace
cargo nextest run --workspace   # if cargo-nextest is installed

# Health API
cargo run -p oracle-server
curl http://127.0.0.1:8080/health

# Fetch sources (point config at running mocks or APIs)
cargo run -p oracle-fetcher -- --config config/sources.example.toml

# Database (for milestone 4+)
docker compose up -d
export DATABASE_URL=postgres://oracle:oracle@localhost:5432/oracle
psql "$DATABASE_URL" -f migrations/001_init.sql
```

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -Dwarnings
```

CI runs build, fmt, clippy, docs, nextest, typos, taplo, markdownlint (including `docs/security/`), yamlfmt, cargo-deny, cargo-audit, secrets grep, and MSRV check on every push/PR.

## Binaries

| Binary | Purpose |
| --- | --- |
| `oracle-server` | REST API (`/health` today) |
| `oracle-fetcher` | Concurrent source fetch CLI |
| `oracle-aggregator` | Aggregation (M2) |
| `oracle-prover` / `oracle-verifier` | ZK (M3) |
| `oracle-submitter` | On-chain submit (M6) |

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
