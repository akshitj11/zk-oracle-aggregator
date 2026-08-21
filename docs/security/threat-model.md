# Threat model

Threat model for the oracle pipeline (M0–M7 implemented). Mainnet gap: replace `MockGroth16Verifier` with a real BN254 Groth16 verifier before trusting on-chain settlement.

## System overview

```mermaid
flowchart TB
  subgraph untrusted [Untrusted]
    APIs[External source APIs]
    StdinAgg[Aggregator stdin JSON]
    ApiCaller[REST API callers]
  end
  subgraph semi [Semi-trusted]
    Prover[oracle-prover]
    Operator[Pipeline operator]
    Postgres[PostgreSQL archive]
  end
  subgraph trusted [Trust assumptions]
    Verifier[oracle-verifier / on-chain vk]
    AggregatorLogic[Published aggregator code]
  end
  APIs --> Fetcher
  Fetcher --> Agg[aggregate]
  StdinAgg --> Agg
  Agg --> Prover
  Prover --> Proof[Groth16 proof]
  Proof --> Verifier
  Proof --> Postgres
  ApiCaller --> Server[oracle-server]
  Proof --> Chain[PredictionMarketOracle M6]
  Chain --> Verifier
  Server --> Fetcher
  Server --> Agg
  Server --> Prover
  Server --> Postgres
```

## Actors

| Actor | Capability | Goal |
| --- | --- | --- |
| **Honest operator** | Runs fetcher, aggregator, prover | Correct market resolution |
| **Malicious source** | Returns biased JSON / wrong outcome | Skew aggregation |
| **Malicious prover** (M3) | Chooses arbitrary private witnesses | Forge proof for wrong outcome |
| **API caller** | Hits `oracle-server` endpoints | DoS, probe internals, trigger bogus resolves |
| **Database attacker** | SQL injection or credential theft | Tamper proof archive or reputation |
| **On-chain user** | Submits `resolveMarket` calldata | Settle market in their favor with a forged proof |

## Trust boundaries

1. **Fetcher → core:** Bodies are untrusted. Parsing and confidence bounds are the gate (F1–F3).
2. **CLI stdin → aggregator:** JSON is untrusted. Bounded read + serde + aggregation invariants must hold without panic.
3. **Aggregator → prover (M3):** Aggregation result is the semantic truth; circuit must prove witness consistency (Z6).
4. **Prover → storage:** `OracleStore` writes proof bytes and `PublicInputs` JSON via parameterized sqlx queries; no string-built SQL.
5. **API → pipeline:** Authenticated `/resolve` triggers fetch → aggregate → prove → store; disputed results return 409.
6. **Off-chain → chain:** `oracle-submitter` encodes calldata; contract verifies proof against on-chain vk; mock verifier is dev-only.
7. **Verifier:** Only trusts vk + public inputs + proof bytes, not the prover or API.

## Attack surfaces

| Surface | Risk | Mitigation |
| --- | --- | --- |
| HTTP response injection | Wrong outcome in JSON | Parse validation; BLAKE3 audit trail |
| Outlier minority source | Skew if threshold wrong | `remove_outliers` at 0.70; disputed flag |
| Empty / single source | Weak consensus | `disputed` when agreement &lt; 0.60 |
| Stdin / body size | Memory DoS | 1 MiB stdin cap; API body limit on `oracle-server` |
| Non-binary witness (M3) | Fake majority | Z1 boolean constraints |
| Witness substitution (M3) | Hide source data | BLAKE3 agreement hash over included `raw_hash` (Z5); Z6 parity with `aggregate()` |
| Tampered proof (M3) | Accept invalid proof | Z7/Z8 Groth16 verify |
| Leaked proving key | Forge arbitrary proofs | G4 gitignore + CI secrets grep |
| SQL injection | Corrupt archive | sqlx parameterized queries in `OracleStore` |
| Unauthenticated `/resolve` | Spam resolutions | `ORACLE_API_KEY` middleware |
| Public input mismatch | On-chain/off-chain drift | `public_inputs_u256` matches Solidity `uint256[2]` order |
| Mock verifier on mainnet | Accept forged proofs | Deploy production Groth16 verifier (C4 policy) |
| Dependency compromise | Supply chain | `cargo audit`, `cargo deny` in CI |

## Composability

| Integration | Note |
| --- | --- |
| PostgreSQL archive | Proofs and source hashes; access control on `DATABASE_URL` |
| REST `/resolve` | Auth + rate limit; 409 on `disputed` |
| Solidity verifier | Public input order must match Rust verifier; mock is not mainnet-safe |
| `oracle-submitter` | Prints calldata without RPC; live broadcast is operator-controlled |

See [adversarial-vectors.md](adversarial-vectors.md) for test mapping and [audit-findings.md](audit-findings.md) for the Phase 1 baseline review.
