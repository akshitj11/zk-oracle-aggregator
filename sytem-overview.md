
```
┌─────────────────────────────────────────────────────────────────┐
│                    ZK ORACLE AGGREGATOR                         │
│                                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ AP News  │  │ Reuters  │  │ Gov Feed │  │  Sports  │         │
│  │   API    │  │   API    │  │   API    │  │   API    │         │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘         │
│       │              │              │              │            │
│       └──────────────┴──────────────┴──────────────┘            │
│                              │                                  │
│                   ┌──────────▼──────────┐                       │
│                   │  Rust Async Fetcher │  ← tokio + reqwest    │
│                   │  (concurrent, with  │                       │
│                   │   timeouts + retry) │                       │
│                   └──────────┬──────────┘                       │
│                              │                                  │
│                   ┌──────────▼──────────┐                       │
│                   │  Aggregation Engine │  ← rust_decimal       │
│                   │  weighted median    │     statrs            │
│                   │  outlier removal    │     dashmap           │
│                   │  consensus check    │                       │
│                   └──────────┬──────────┘                       │
│                              │                                  │
│                   ┌──────────▼──────────┐                       │
│                   │   ZK Circuit        │  ← ark-groth16        │
│                   │   (Groth16/BN254)   │     ark-bn254         │
│                   │   prove: N sources  │     blake3            │
│                   │   agreed honestly   │                       │
│                   └──────────┬──────────┘                       │
│                              │                                  │
│              ┌───────────────┴────────────────┐                 │
│              │                                │                 │
│   ┌──────────▼──────────┐      ┌──────────────▼──────────┐      │
│   │  ZK Proof (128B)    │      │  Smart Contract          │     │
│   │  stored on-chain    │─────▶│  verifies proof          │     │
│   │  verifiable forever │      │  auto-pays winners       │     │
│   └─────────────────────┘      └──────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
