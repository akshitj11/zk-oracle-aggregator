# Release checklist

Run `./scripts/ci-local.sh` with Postgres up and `DATABASE_URL` set. Confirm GitHub CI is green on `main`. Apply `migrations/001_init.sql` and `migrations/002_indexes.sql` on the target database. Set `ORACLE_API_KEY`, `SOURCES_CONFIG`, and proving key paths from an offline ceremony (`scripts/trusted-setup.sh`). Deploy `contracts/` with a production Groth16 verifier, not `MockGroth16Verifier`. Point `oracle-submitter` at Sepolia or mainnet RPC with a funded key. Enable private GitHub contributions if using a private fork for heatmap tracking.
