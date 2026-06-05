# Architecture

## System overview

```mermaid
flowchart TB
  AP[Source APIs] --> Fetcher
  Fetcher[Rust async fetcher] --> Agg[Aggregation engine]
  Agg --> Circuit[ZK circuit Groth16]
  Circuit --> Proof[Proof archive]
  Circuit --> Chain[Smart contract verify]
  Proof --> API[REST API]
```

## Workspace layout

| Component | Crate / binary | Status |
| --- | --- | --- |
| Core library | `oracle-core` | Active |
| HTTP fetcher | `oracle-fetcher` | Active |
| API server | `oracle-server` | Health only |
| Aggregator CLI | `oracle-aggregator` | Active |
| Prover / verifier | `oracle-prover`, `oracle-verifier` | Planned (M3) |
| Chain submitter | `oracle-submitter` | Planned (M6) |

## Aggregation pipeline (implemented)

1. Input: `Vec<SourceResponse>` (JSON on stdin for CLI).
2. `remove_outliers` at threshold `0.70`.
3. If agreement ratio &lt; `0.60`, mark `disputed`.
4. `weighted_median` on remaining sources (confidence as weight).
5. Output: `AggregationResult` with `rust_decimal` fields for reproducibility.

## Fetch pipeline (implemented)

1. Load source list from TOML config (`id`, `url`).
2. `fetch_all_sources` runs concurrent HTTP GETs with timeout and retries.
3. Each body is hashed with BLAKE3; JSON is parsed to `SourceResponse`.
4. Failed sources are omitted (no panic on partial failure).

## Data types

- `Outcome`: `YES` | `NO` | `UNKNOWN`
- `SourceResponse`: `source_id`, `outcome`, `confidence`, `fetched_at`, `raw_hash`

## Storage (planned M4)

PostgreSQL schema in `migrations/001_init.sql`: `oracle_proofs`, `source_responses`, `source_reputation`.

## Local services

```bash
docker compose up -d   # Postgres on localhost:5432
```

`DATABASE_URL=postgres://oracle:oracle@localhost:5432/oracle`
