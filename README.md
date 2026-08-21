# ZK Oracle Aggregator

[![CI](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml/badge.svg)](https://github.com/akshitj11/zk-oracle-aggregator/actions/workflows/ci.yaml)

Prediction markets settle on oracle output. Token-vote oracles let one whale pick the winner. This repo replaces that vote with multi-source consensus and a Groth16 proof on BN254. M0–M7 are implemented.

## System overview

Sixteen HTTP feeds, outlier removal at 0.70 peer agreement, weighted median consensus, Groth16 proof on BN254. Solid arrows are `POST /resolve` through `oracle-server`. Dashed arrows are the standalone CLI path for local prove/verify.

```mermaid
flowchart TB
  subgraph config [Config]
    SourcesTOML["sources.toml: up to 16 source URLs"]
    EnvConfig["DATABASE_URL ORACLE_API_KEY Groth16 pk/vk"]
  end

  subgraph external [Untrusted]
    ExtApis["External APIs: outcome confidence JSON"]
    ApiClients["REST callers on /resolve"]
  end

  subgraph core [oracle-core library]
    FetchMod["fetcher: 16 concurrent GETs 5s timeout BLAKE3 raw_hash"]
    AggMod["aggregator: outlier 0.70 weighted median disputed lt 0.60"]
    CircuitMod["circuit: 16-slot R1CS binary outcomes inclusion flags"]
    ProverMod["prover: Groth16 BN254 witness from aggregate"]
    VerifierMod["verifier: vk public inputs proof bytes only"]
    StoreMod["OracleStore: sqlx oracle_proofs source_responses reputation"]
    ChainMod["chain: public_inputs_u256 resolveMarket calldata"]
  end

  subgraph clis [CLI debug path]
    FetchCli["oracle-fetcher"]
    AggCli["oracle-aggregator"]
    ProverCli["oracle-prover"]
    VerifierCli["oracle-verifier"]
  end

  subgraph server [oracle-server REST]
    Middleware["middleware: API key rate limit 1 MiB body"]
    ResolveRoute["POST /resolve: fetch aggregate prove store"]
    ReadRoutes["GET /proof /verify /reputation /health /metrics"]
  end

  subgraph db [PostgreSQL]
    ProofsTbl["oracle_proofs: proof_bytes public_inputs market_id"]
    ResponsesTbl["source_responses: raw_hash linked to proof_id"]
    RepTbl["source_reputation: accuracy weights"]
  end

  subgraph chainLayer [On-chain M6]
    Submitter["oracle-submitter: calldata or ETH_RPC_URL broadcast"]
    OracleContract["PredictionMarketOracle.resolveMarket"]
    MockVk["MockGroth16Verifier: dev XOR check only"]
    ProdVk["BN254 Groth16 vk: mainnet ceremony match"]
  end

  SourcesTOML --> FetchMod
  SourcesTOML -.-> FetchCli
  EnvConfig --> ResolveRoute
  EnvConfig --> StoreMod
  ExtApis --> FetchMod
  ApiClients --> Middleware
  Middleware --> ResolveRoute
  Middleware --> ReadRoutes
  ResolveRoute --> FetchMod
  FetchMod --> AggMod
  AggMod -->|"disputed: 409"| ApiClients
  CircuitMod --> ProverMod
  AggMod --> ProverMod
  ProverMod --> VerifierMod
  ProverMod --> StoreMod
  StoreMod --> ProofsTbl
  StoreMod --> ResponsesTbl
  StoreMod --> RepTbl
  ReadRoutes --> ProofsTbl
  ReadRoutes --> VerifierMod
  ProverMod --> ChainMod
  ChainMod --> Submitter
  Submitter --> OracleContract
  OracleContract --> MockVk
  MockVk -.->|"mainnet: swap vk"| ProdVk
  FetchCli -.-> AggCli
  AggCli -.-> ProverCli
  ProverCli -.-> VerifierCli
  FetchCli -.-> FetchMod
  AggCli -.-> AggMod
  ProverCli -.-> ProverMod
  VerifierCli -.-> VerifierMod
```

Disputed markets return 409 before proving. Nothing writes to Postgres. Verifiers only check vk, public inputs, and proof bytes. The operator is not in that trust set. `MockGroth16Verifier` is dev-only; mainnet needs a real BN254 Groth16 vk from the same ceremony as Rust pk/vk.

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
