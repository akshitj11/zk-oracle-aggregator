# Why this project exists

Prediction market contracts cannot see the real world. They depend on an **oracle** to report outcomes. When that report is wrong, funds settle to the wrong side.

## Traditional oracle (governance trust)

```mermaid
flowchart TB
  M1[Market question]
  V1[Token vote or committee]
  C1[Contract trusts reported outcome]
  M1 --> V1 --> C1
```

A concentrated voter or compromised committee can report a false result. The contract has no way to verify multi-source agreement or honest computation.

## ZK oracle aggregator (cryptographic verification)

```mermaid
flowchart TB
  M2[Market question]
  S2[N independent sources]
  A2[Honest aggregation]
  P2[Groth16 proof]
  V2[Verifier - math not votes]
  C2[Contract settles only if proof valid]
  M2 --> S2 --> A2 --> P2 --> V2 --> C2
```

This system fetches from many sources, aggregates with outlier removal and weighted consensus, and produces a **short proof** that the computation was run correctly. Anyone can verify the proof; invalid proofs are rejected.

## Comparison

| | Traditional oracle | ZK oracle aggregator |
| --- | --- | --- |
| Trust basis | Governance, votes | Math + multi-source consensus |
| Dispute | Slow human process | `disputed` flag, no proof issued |
| Audit | Opaque logs | Public proofs + source hashes |
| On-chain | Often none | Groth16 verify on-chain (phase B) |

## Milestone progress

```mermaid
flowchart LR
  M0[M0 scaffold + CI] --> M1[M1 multi-source fetch]
  M1 --> M2[M2 aggregation]
  M2 --> M3[M3 ZK prove]
  M3 --> M4[M4 proof archive]
  M4 --> M5[M5 REST API]
  M5 --> M6[M6 on-chain Sepolia]
```

**Current:** M0–M1 (fetcher, health API, CI). M2–M6 planned.
