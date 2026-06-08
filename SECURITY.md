# Security policy

## Supported versions

| Version | Supported |
| --- | --- |
| `main` branch | Yes |
| Tagged releases | Yes |
| Other branches | No |

## Reporting a vulnerability

If you believe you have found a security issue in **zk-oracle-aggregator**:

1. **Do not** open a public GitHub issue for exploitable vulnerabilities.
2. Email **akshitcasual1@gmail.com** with:
   - Description of the issue
   - Steps to reproduce
   - Impact assessment (especially oracle manipulation or proof forgery)
   - Affected commit or version if known
3. Allow up to **7 days** for an initial response.

We will coordinate disclosure timing with you after confirming the issue.

## Scope

In scope:

- `crates/oracle-core` (fetcher, aggregator, future ZK modules)
- Workspace binaries (`oracle-fetcher`, `oracle-aggregator`, `oracle-prover`, `oracle-verifier`, etc.)
- Incorrect aggregation, proof verification bypass, or witness forgery

Out of scope (unless directly affecting this repo):

- Third-party source APIs
- GitHub Actions infrastructure
- Dependencies without a demonstrated exploit path in this codebase

## Security documentation

- [Invariants](docs/security/invariants.md)
- [Threat model](docs/security/threat-model.md)
- [Adversarial test matrix](docs/security/adversarial-vectors.md)

## Hardening

CI runs `cargo audit`, `cargo deny`, clippy with `-Dwarnings`, and adversarial integration tests. See [README.md](README.md#development).
