# Audit skills setup

## Local clones (storage)

```text
/home/joshi/storage/cursor-skills/pashov-skills/     # x-ray, solidity-auditor
/home/joshi/storage/cursor-skills/web3-skills/       # client-auditor, contract-auditor
```

## Cursor install prompts

```
Install https://github.com/pashov/skills/
Install skills in https://github.com/DarkNavySecurity/web3-skills/
```

## Project skill

[`.cursor/skills/oracle-security/SKILL.md`](../../.cursor/skills/oracle-security/SKILL.md) — always active for this repo.

## Suggested workflow

| When | Command |
| --- | --- |
| Phase 1 baseline | x-ray → `docs/security/x-ray/` |
| Before M3 | `/client-auditor start crates/oracle-core` |
| After M3 | `/client-auditor verify crates/oracle-core deep` |
| M6 Solidity | pashov `solidity-auditor`, DarkNavy `contract-auditor` |
