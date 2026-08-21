# ZK Oracle Aggregator

[![CI](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml/badge.svg)](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml)

Prediction markets settle on oracle output. Token-vote oracles let one whale pick the winner. This repo fetches up to 16 independent feeds, aggregates with outlier removal and weighted median, proves the computation in Groth16 (BN254), archives proofs in Postgres, serves REST resolve, and settles on-chain through a mock Solidity verifier. M0–M7 are implemented. Mainnet still needs a real BN254 verifier, live RPC broadcast from `oracle-submitter`, and non-mock source URLs.

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

`oracle-fetcher` hits configured URLs concurrently (max 16 sources). `oracle-aggregator` reads `SourceResponse[]` JSON from stdin. `oracle-prover` reads the same JSON, builds a witness from `aggregate()`, and prints a Groth16 proof with public inputs. `oracle-verifier` checks proof JSON on stdin. `oracle-server` exposes `GET /health`, `GET /proof/:market_id`, `GET /reputation/:source_id`, `GET /verify/:market_id`, and `POST /resolve`. Set `DATABASE_URL`, optional `ORACLE_API_KEY`, and `SOURCES_CONFIG`. `oracle-submitter` builds `resolveMarket` calldata from proof JSON; without `ETH_RPC_URL` it prints calldata only.

Prove and verify locally:

```bash
cargo run -p oracle-fetcher -- --config config/sources.integration.toml > /tmp/responses.json
cargo run -p oracle-prover -- --write-proving-key /tmp/pk.bin --write-verifying-key /tmp/vk.bin \
  < /tmp/responses.json > /tmp/proof.json
cargo run -p oracle-verifier -- --verifying-key /tmp/vk.bin < /tmp/proof.json
```

Further reading: [docs/WHY.md](docs/WHY.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/security/invariants.md](docs/security/invariants.md).

## API

Start Postgres, apply migrations, then run the server:

```bash
export DATABASE_URL=postgres://oracle:oracle@localhost:5432/oracle
export ORACLE_API_KEY=dev-key
cargo run -p oracle-server
curl http://127.0.0.1:8080/health
curl -H "X-API-Key: dev-key" -X POST http://127.0.0.1:8080/resolve \
  -H 'Content-Type: application/json' \
  -d '{"market_id":"0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"}'
```

`POST /resolve` returns `409` when aggregation is disputed. Proofs persist in Postgres and are readable via `GET /proof/:market_id` (hex-encoded market id).

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).
