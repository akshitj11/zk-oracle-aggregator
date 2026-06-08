# Security invariants

Statements that must hold for the oracle pipeline to be trustworthy. Status: **implemented** (M0–M2) or **planned** (M3 ZK).

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

## ZK layer (planned — Phase 2 / M3)

| ID | Invariant | Status |
| --- | --- | --- |
| Z1 | Every constrained outcome variable in-circuit is binary (0 or 1) | planned |
| Z2 | Excluded sources (`included = 0`) contribute zero effective weight in-circuit | planned |
| Z3 | Public `source_count` equals sum of inclusion flags | planned |
| Z4 | Public `final_outcome` matches weighted majority of included sources | planned |
| Z5 | Each public commitment equals Poseidon(outcome, confidence, raw_hash) for that source | planned |
| Z6 | Witness builder output matches M2 `aggregate()` for honest runs; mismatch is an error | planned |
| Z7 | Groth16 verify succeeds only with correct vk and canonical public input order | planned |
| Z8 | Tampered proof bytes or public inputs cause verify to fail | planned |

## Entry points (audit scope)

| Component | Entry | Trust |
| --- | --- | --- |
| `oracle-fetcher` | HTTP GET to configured URLs | Untrusted network |
| `oracle-aggregator` | stdin JSON `Vec<SourceResponse>` | Untrusted until validated |
| `oracle-server` | `GET /health` | Public read |
| `oracle-prover` / `oracle-verifier` | CLI (M3) | Prover is semi-trusted; verifier is trustless |
