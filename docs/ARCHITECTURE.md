# Architecture

The pipeline is fetch, aggregate, prove, store, serve, settle. M0–M7 are implemented in this repo. Mainnet still requires a production Groth16 verifier, live RPC submitter, and real source URLs.

```mermaid
flowchart TB
  AP[Source APIs] --> Fetcher
  Fetcher[Rust async fetcher] --> Agg[Aggregation engine]
  Agg --> Circuit[ZK circuit Groth16]
  Circuit --> Proof[Proof archive]
  Circuit --> Chain[Smart contract verify]
  Proof --> API[REST API]
```

## Workspace

| Component | Crate / binary | Status |
| --- | --- | --- |
| Core library | `oracle-core` | Active |
| HTTP fetcher | `oracle-fetcher` | Active |
| API server | `oracle-server` | Active (M5) |
| Aggregator CLI | `oracle-aggregator` | Active |
| Prover / verifier | `oracle-prover`, `oracle-verifier` | Active (M3) |
| Chain submitter | `oracle-submitter` | Active (M6 mock calldata) |

## Aggregation (implemented)

Input is `Vec<SourceResponse>` (stdin JSON in the CLI). `remove_outliers` drops sources below 30% peer agreement (threshold 0.70). If fewer than 60% of inputs survive, `disputed` is true. Otherwise `weighted_median` compares total Yes vs No confidence weight. Output uses `rust_decimal` so the same bytes reproduce on every machine.

## ZK prove (M3)

`build_witness` calls `aggregate()` and rejects disputed markets. Witnesses pad to `MAX_SOURCES` (16) with zero weight on unused slots. The R1CS enforces binary outcomes, inclusion flags, source count, and weighted majority via a bit-decomposed margin witness. `prove_responses` in `oracle-core` runs Groth16 setup (or loads serialized pk/vk), proves, and returns `OracleProof` plus `PublicInputs`. Verifiers check two public field elements: final outcome and included source count. Agreement over included `raw_hash` values is committed off-circuit with BLAKE3 (Z5).

## Fetch (implemented)

Sources load from TOML (`id`, `url`). `fetch_all_sources_with_limit` runs concurrent GETs with 5s timeout and two retries, caps at `MAX_SOURCES` (16), hashes each body with BLAKE3, parses `{outcome, confidence}` JSON. Failed HTTP or parse drops that source without panicking.

## Types

`Outcome` is YES, NO, or UNKNOWN. `SourceResponse` carries `source_id`, `outcome`, `confidence`, `fetched_at`, and `raw_hash`.

## Storage (implemented)

`OracleStore` in `oracle-core::storage` uses sqlx against Postgres. `save_proof` and `get_proof` round-trip `oracle_proofs` by `market_id` BYTEA. `save_source_responses` links fetch rows to `proof_id`. `update_reputation` and `get_reputation` maintain `source_reputation` accuracy weights. Schema lives in `migrations/001_init.sql` with indexes in `migrations/002_indexes.sql`. CI and local tests require `DATABASE_URL`; compile-time query checks use the committed `.sqlx/` offline cache.

## REST API (implemented)

`oracle-server` loads `AppState` with `OracleStore`, Groth16 keys, and source URLs from TOML. `POST /resolve` runs fetch, `aggregate()`, rejects disputed markets with `409`, then `prove_responses()` and storage. `GET /proof/:market_id` returns stored proofs. `GET /verify/:market_id` re-runs Groth16 verify. Optional `ORACLE_API_KEY` gates mutating routes; rate limiting and 1 MiB body caps apply via middleware.

## On-chain (M6)

`contracts/` ships `PredictionMarketOracle` and `MockGroth16Verifier`. Foundry tests cover valid proof acceptance, invalid proof rejection, and double-resolve guards. `oracle-core::chain` encodes `public_inputs_u256` and mock proof components for dev networks. `oracle-submitter` reads proof JSON and prints or broadcasts `resolveMarket` calldata. Swap `MockGroth16Verifier` for a BN254 Groth16 verifier generated from the same ceremony as Rust `pk`/`vk` before mainnet.

## Production (M7)

`config/sources.prod.example.toml`, `scripts/trusted-setup.sh`, `/metrics` stub, graceful shutdown, and `docs/RELEASE.md` cover operator hardening. Grafana dashboards and live Sepolia broadcast are not shipped yet.

## Local Postgres

```bash
docker compose up -d
export DATABASE_URL=postgres://oracle:oracle@localhost:5432/oracle
```

## Delivery

M0–M7 landed on `main`. Remaining production work: real verifier, live submitter, prod source URLs, observability dashboards. Run `./scripts/ci-local.sh` before each commit.
