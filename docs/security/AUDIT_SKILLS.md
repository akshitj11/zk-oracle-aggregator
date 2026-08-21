# Audit skills setup

Audit tooling runs from local Cursor skills only (`.cursor/skills/`, not committed).

## Project skill

Use the oracle-security skill in `.cursor/skills/oracle-security/SKILL.md` before editing fetcher, aggregator, or security docs.

## Suggested workflow

| When | Action |
| --- | --- |
| Every PR | Run adversarial tests + full CI locally |
| Before M3 | Deep review of ZK invariants Z1–Z8 |
| After M3 | Re-run adversarial ZK tests |
| M6 Solidity | Contract review + Foundry invalid-proof test |

Findings land in `docs/security/audit-findings.md` with invariant IDs.
