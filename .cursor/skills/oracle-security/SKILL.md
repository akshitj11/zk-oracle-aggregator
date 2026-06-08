---
name: oracle-security
description: Security review for zk-oracle-aggregator Rust code. Use when editing crates/oracle-core, bins/, docs/security/, CI workflows, or aggregation/fetcher logic. Enforces invariants, adversarial tests, and no unwrap in production paths.
---

# Oracle security skill

## Before coding

1. Read [docs/security/invariants.md](../../docs/security/invariants.md)
2. Check [docs/security/adversarial-vectors.md](../../docs/security/adversarial-vectors.md) for required tests
3. If changing trust boundaries, update [docs/security/threat-model.md](../../docs/security/threat-model.md)

## Rules

- **No panic on untrusted input** — stdin JSON, HTTP bodies, CLI args must return `Result` or omit source
- **Adversarial tests** — every security-sensitive change needs a negative test in `crates/oracle-core/tests/security_adversarial.rs` or module unit tests
- **Avoid `unwrap`/`expect`** in `oracle-core` production code (tests may use them)
- **Disputed markets** — never treat `disputed: true` as final settlement
- **Secrets** — never commit `keys/`, `*.pk`, `vk.bin`, API keys

## External audit skills

Clones on storage (install in Cursor if needed):

- x-ray: `/home/joshi/storage/cursor-skills/pashov-skills/` — `run x-ray on the codebase`
- client-auditor: `/home/joshi/storage/cursor-skills/web3-skills/client-auditor/` — `/client-auditor start crates/oracle-core`

## ZK phase (M3)

When `zk/` exists: read Z1–Z8 invariants; add one adversarial test per gadget; single `public_inputs()` source of truth.

## PR checklist

Use [.github/pull_request_template.md](../../.github/pull_request_template.md).
