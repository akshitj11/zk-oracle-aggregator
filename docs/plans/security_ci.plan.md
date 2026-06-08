# Phase 1 — Security and CI hardening

**Status:** complete on `main`. M3 ZK may start at `deps` — see [m3_zk_circuit.plan.md](m3_zk_circuit.plan.md).

## Goal

Audit-ready foundations: security docs, audit skills, CI gates, M2 adversarial tests. No arkworks in this phase.

## Steps

1. **Security artifacts** — `docs/security/*`, `SECURITY.md`, PR template
2. **Audit skills** — pashov x-ray, DarkNavy client-auditor, `.cursor/skills/oracle-security/`
3. **CI** — markdownlint on security docs, secrets grep, `security_adversarial.rs`
4. **client-auditor** baseline on `crates/oracle-core`
5. **Verify** — `cargo test --workspace`, update M3 prerequisite

## Source repos

- [pashov/skills](https://github.com/pashov/skills) (x-ray)
- [DarkNavySecurity/web3-skills](https://github.com/DarkNavySecurity/web3-skills) (client-auditor)
- [marchev/awesome-ai-web3-security](https://github.com/marchev/awesome-ai-web3-security)

## Done when

See [docs/security/](../security/) and verification section in the Cursor plan (Phase 1 complete).

**Next:** M3 ZK at `deps` sub-layer.
