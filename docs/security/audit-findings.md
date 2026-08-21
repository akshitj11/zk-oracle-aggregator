# Audit findings (client-auditor baseline)

Phase 1 review of `crates/oracle-core` (M0–M2). Methodology adapted from [DarkNavy client-auditor](https://github.com/DarkNavySecurity/web3-skills/tree/main/client-auditor).

## Summary

| Severity | Open | Accepted |
| --- | --- | --- |
| Critical | 0 | 0 |
| High | 0 | 0 |
| Medium | 0 | 1 |
| Low | 0 | 2 |
| Info | 0 | 2 |

No Medium+ open findings. Phase 1 complete for audit gate.

---

## I-001 — Parse errors silently dropped on fetch

| Field | Value |
| --- | --- |
| Severity | Info |
| Status | accepted |
| Location | `fetcher/mod.rs` — `fetch_source` |

`parse_response` failure returns `None` without logging. Operator cannot distinguish HTTP failure from invalid JSON.

**Mitigation (deferred):** add `tracing` warn on parse failure in M4+ observability work.

---

## I-002 — `decimal_from_f64` uses `unwrap_or`

| Field | Value |
| --- | --- |
| Severity | Info |
| Status | accepted |
| Location | `aggregator/mod.rs` |

Non-finite floats map to `Decimal::ZERO`. Confidence inputs are validated as finite f64 at fetch boundary.

---

## L-001 — Unbounded stdin JSON in aggregator CLI

| Field | Value |
| --- | --- |
| Severity | Low |
| Status | accepted |
| Location | `bins/oracle-aggregator/src/main.rs` |

Malicious local user can supply very large JSON and exhaust memory.

**Mitigation:** CLI is operator-trusted in M2; M5 API will add size limits. Documented in [threat-model.md](threat-model.md).

---

## L-002 — No explicit max source count on aggregation input

| Field | Value |
| --- | --- |
| Severity | Low |
| Status | accepted |
| Location | `aggregator::aggregate` |

Unbounded `Vec` length is O(n) per aggregation. ZK phase will use `MAX_SOURCES = 16`.

**Mitigation:** enforce cap in witness builder at M3; optional CLI limit in M5.

---

## M-001 — Disputed results not blocked at CLI

| Field | Value |
| --- | --- |
| Severity | Medium |
| Status | accepted (policy) |
| Location | `bins/oracle-aggregator` |

CLI prints `disputed: true` results without non-zero exit. Downstream could misinterpret output.

**Mitigation:** invariant G3; `POST /resolve` returns `409` on disputed markets. Prover refuses disputed witnesses (Z6).

---

## M6 on-chain gate (2026-08-21)

| Check | Result |
| --- | --- |
| Invalid proof reverts | Foundry `testInvalidProofReverts` |
| Double resolve reverts | Foundry `testDoubleResolveReverts` |
| Public input binding | on-chain check matches Rust order |
| Mock verifier PoC | `MockGroth16Verifier` for Sepolia dev |

Production mainnet requires a BN254 Groth16 verifier generated from the production trusted setup, not the mock XOR verifier.

---

## M5 API gate (2026-08-21)

| Check | Result |
| --- | --- |
| Auth middleware on `/resolve` | implemented (`ORACLE_API_KEY`) |
| Rate limit | implemented (token bucket) |
| Body size cap | implemented (1 MiB) |
| Disputed → 409 | implemented |
| Integration tests | `bins/oracle-server/tests/api_integration.rs` |

---

## M7 regression re-audit (2026-08-21)

Full pipeline M0–M6 re-checked against Phase 1 findings. No regressions in fetcher or aggregator invariants. New surfaces (storage, API, chain) covered by sqlx parameterization, API integration tests, and Foundry invalid-proof tests. Remaining production gap: replace mock verifier before mainnet.

---

- Confidence range validation on parse (F2)
- BLAKE3 body hash (F1)
- Empty input → disputed (A2)
- Adversarial tests in `tests/security_adversarial.rs`
- CI: `cargo audit`, `cargo deny`, secrets grep
