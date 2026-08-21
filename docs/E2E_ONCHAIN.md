# On-chain settlement (M6)

Fetch configured sources, aggregate, prove with `oracle-prover`, persist via `oracle-server`, then submit calldata with `oracle-submitter`.

```bash
cargo run -p oracle-fetcher -- --config config/sources.integration.toml > /tmp/responses.json
cargo run -p oracle-prover < /tmp/responses.json > /tmp/proof.json
export DATABASE_URL=postgres://oracle:oracle@localhost:5432/oracle
export ORACLE_API_KEY=dev-key
cargo run -p oracle-server &
curl -H "X-API-Key: dev-key" -X POST http://127.0.0.1:8080/resolve \
  -H 'Content-Type: application/json' \
  -d '{"market_id":"0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"}'
cargo run -p oracle-submitter -- --proof /tmp/proof.json \
  --market-id 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20 \
  --outcome
```

Sepolia deploy uses Foundry:

```bash
cd contracts
forge install foundry-rs/forge-std
export ETH_RPC_URL=https://sepolia.example
export PRIVATE_KEY=0x...
forge script script/Deploy.s.sol --rpc-url "$ETH_RPC_URL" --broadcast
```

The mock verifier accepts proofs whose `c[0]` matches the XOR commitment of public inputs. Production deployments must swap in a BN254 Groth16 verifier generated from the same trusted setup as `oracle-prover`.
