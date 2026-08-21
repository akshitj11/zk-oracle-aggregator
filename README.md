# ZK Oracle Aggregator

[![CI](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml/badge.svg)](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml)

Prediction markets settle on oracle output. When that output comes from a token vote, one whale can pick the winner. This repo fetches N independent feeds, aggregates with outlier removal and a weighted median, and will attach a Groth16 proof (BN254) so verification does not depend on trusting the operator. Fetch and aggregate (M0–M2) run today; proof generation, Postgres archive, REST resolve, and on-chain verify are in progress.

## Quick start

Rust 1.88+ and Docker for Postgres. Run `./scripts/ci-local.sh` before every commit; it mirrors GitHub CI.

```bash
cargo build --workspace
cargo test --workspace
cargo run -p oracle-server
curl http://127.0.0.1:8080/health
cargo run -p oracle-fetcher -- --config config/sources.integration.toml
docker compose up -d
export DATABASE_URL=postgres://oracle:oracle@localhost:5432/oracle
psql "$DATABASE_URL" -f migrations/001_init.sql
```

## Development

`cargo fmt --all`, `cargo clippy --workspace --all-targets -- -Dwarnings`, and `./scripts/ci-local.sh` must pass locally before push. GitHub runs the same checks on every PR to `main`.

## Binaries

`oracle-fetcher` hits configured URLs concurrently (max 16 sources). `oracle-aggregator` reads `SourceResponse[]` JSON from stdin. `oracle-server` exposes `/health` today. `oracle-prover`, `oracle-verifier`, and `oracle-submitter` are stubs until M3 and M6.

Further reading: [docs/WHY.md](docs/WHY.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/security/invariants.md](docs/security/invariants.md).

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).
