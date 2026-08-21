# ZK Oracle Aggregator

[![CI](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml/badge.svg)](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml)

Prediction markets settle on oracle output. When that output comes from a token vote, one whale can pick the winner. This repo fetches N independent feeds, aggregates with outlier removal and a weighted median, and attaches a Groth16 proof (BN254) so verification does not depend on trusting the operator. Fetch, aggregate, and prove (M0–M3) run in `oracle-core` and the prover CLIs. Postgres archive, REST resolve, and on-chain verify are in progress (M4–M6).

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

`oracle-fetcher` hits configured URLs concurrently (max 16 sources). `oracle-aggregator` reads `SourceResponse[]` JSON from stdin. `oracle-prover` reads the same JSON, builds a witness from `aggregate()`, and prints a Groth16 proof with public inputs. `oracle-verifier` checks proof JSON on stdin. `oracle-server` exposes `/health` today. `oracle-submitter` is a stub until M6.

Prove and verify locally:

```bash
cargo run -p oracle-fetcher -- --config config/sources.integration.toml > /tmp/responses.json
cargo run -p oracle-prover -- --write-proving-key /tmp/pk.bin --write-verifying-key /tmp/vk.bin \
  < /tmp/responses.json > /tmp/proof.json
cargo run -p oracle-verifier -- --verifying-key /tmp/vk.bin < /tmp/proof.json
```

Further reading: [docs/WHY.md](docs/WHY.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/security/invariants.md](docs/security/invariants.md).

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).
