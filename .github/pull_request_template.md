## Summary

<!-- What changed and why -->

## Security checklist

- [ ] I read [docs/security/invariants.md](../docs/security/invariants.md) for affected components
- [ ] New or changed invariants are documented (or marked `planned` for M3)
- [ ] Adversarial / negative tests added for security-sensitive behavior
- [ ] No secrets committed (`keys/`, `*.pk`, `vk.bin`, API keys)
- [ ] `cargo test --workspace` passes locally
- [ ] Dependency changes reviewed (`cargo deny` / `cargo audit` if deps changed)

## Test plan

<!-- How you verified the change -->
