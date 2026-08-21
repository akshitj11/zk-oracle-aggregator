# Security invariants

Statements that must hold for the oracle pipeline to be trustworthy. M0–M7 invariants below are **implemented** unless marked **policy** (operator obligation, not enforced in code).

## Global

| ID | Invariant | Status |
| --- | --- | --- |
| G1 | Aggregation output is deterministic for a fixed `Vec<SourceResponse>` input | implemented |
| G2 | Failed HTTP fetches never panic the fetcher; they are omitted from results | implemented |
| G3 | Disputed markets (`disputed: true`) must not be treated as final settlement without human review | implemented |
| G4 | Proving keys (`pk`) and toxic setup material are never committed to git | implemented (policy) |

## Fetcher (`oracle-core::fetcher`)

| ID | Invariant | Status |
| --- | --- | --- |
| F1 | `SourceResponse.raw_hash` is BLAKE3 of the raw HTTP body at fetch time | implemented |
| F2 | `confidence` is in `[0.0, 1.0]` after `parse_response`; out-of-range JSON is rejected | implemented |
| F3 | `Outcome` at parse boundary is one of `YES`, `NO`, `UNKNOWN` (serde enum) | implemented |
| F4 | `fetched_at` is a Unix timestamp (seconds) | implemented |

## Aggregator (`oracle-core::aggregator`)

| ID | Invariant | Status |
| --- | --- | --- |
| A1 | `remove_outliers` keeps only sources whose outcome agrees with at least 30% of peers (threshold `0.70`) | implemented |
| A2 | `aggregate` on empty input returns `disputed: true`, `source_count: 0`, `Outcome::Unknown` | implemented |
| A3 | `aggregate` marks `disputed` when agreement ratio &lt; 0.60 (or input empty) | implemented |
| A4 | `weighted_median` uses only Yes/No weights; Unknown sources contribute zero to yes/no totals | implemented |
| A5 | `weighted_median` returns `Unknown` when total Yes+No weight is zero | implemented |

## ZK layer (M3)

| ID | Invariant | Status |
| --- | --- | --- |
| Z1 | Every constrained outcome variable in-circuit is binary (0 or 1) | implemented |
| Z2 | Excluded sources (`included = 0`) contribute zero effective weight in-circuit | implemented |
| Z3 | Public `source_count` equals sum of inclusion flags | implemented |
| Z4 | Public `final_outcome` matches weighted majority of included sources | implemented |
| Z5 | Agreement hash is BLAKE3 over included sources' `raw_hash` values (off-circuit commitment) | implemented |
| Z6 | Witness builder output matches M2 `aggregate()` for honest runs; mismatch is an error | implemented |
| Z7 | Groth16 verify succeeds only with correct vk and canonical public input order | implemented |
| Z8 | Tampered proof bytes or public inputs cause verify to fail | implemented |

## Storage (M4)

| ID | Invariant | Status |
| --- | --- | --- |
| S1 | `OracleStore` uses parameterized sqlx queries; no string-built SQL | implemented |
| S2 | `get_proof` round-trips `proof_bytes` and `PublicInputs` for a `market_id` | implemented |
| S3 | `save_source_responses` stores per-source `raw_hash` linked to `proof_id` | implemented |
| S4 | `update_reputation` / `get_reputation` maintain bounded `current_weight` in `[0, 1]` | implemented |

## API (M5)

| ID | Invariant | Status |
| --- | --- | --- |
| P1 | `POST /resolve` without valid `ORACLE_API_KEY` returns 401 | implemented |
| P2 | Disputed aggregation returns 409 and does not persist a proof | implemented |
| P3 | Request bodies are capped at 1 MiB | implemented |
| P4 | Rate limit exhaustion returns 429 | implemented |
| P5 | `GET /verify/:market_id` re-runs Groth16 verify on stored proof bytes | implemented |

## On-chain (M6)

| ID | Invariant | Status |
| --- | --- | --- |
| C1 | `public_inputs_u256` encodes `(outcome, source_count)` in canonical order for Solidity | implemented |
| C2 | `MockGroth16Verifier` accepts only proofs whose `c[0]` matches the XOR commitment of public inputs | implemented |
| C3 | `PredictionMarketOracle` rejects invalid proofs and double resolve on the same `market_id` | implemented |
| C4 | Production deployments use a BN254 Groth16 verifier from the same ceremony as Rust `pk`/`vk` | policy |

## Entry points (audit scope)

| Component | Entry | Trust |
| --- | --- | --- |
| `oracle-fetcher` | HTTP GET to configured URLs | Untrusted network |
| `oracle-aggregator` | stdin JSON `Vec<SourceResponse>` | Untrusted until validated |
| `oracle-server` | REST (`/health` public; `/resolve` authenticated) | Untrusted callers on mutating routes |
| `oracle-prover` / `oracle-verifier` | CLI | Prover is semi-trusted; verifier is trustless |
| `oracle-submitter` | proof JSON + RPC (optional) | Operator-trusted calldata builder |
| `PredictionMarketOracle` | `resolveMarket` calldata | Untrusted submitters; contract trusts vk only |
