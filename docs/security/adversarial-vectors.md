# Adversarial vectors

Attack scenarios mapped to tests. **implemented** = covered today; **planned** = M3 ZK phase.

## Fetcher

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Invalid JSON body | `parse_response` returns `InvalidJson` | `security_adversarial::parse_rejects_invalid_json` | implemented |
| Confidence &gt; 1.0 | `InvalidConfidence` | `fetcher::parse_rejects_invalid_confidence` | implemented |
| Confidence &lt; 0.0 | `InvalidConfidence` | `security_adversarial::parse_rejects_negative_confidence` | implemented |
| HTTP 500 / timeout | Source omitted from results | `fetcher::fetch_all_sources_skips_failed` | implemented |
| Body hash mismatch (audit) | Off-chain compare `raw_hash` vs recomputed BLAKE3 | manual / M4 storage | planned |

## Aggregator

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Empty input | `disputed: true`, zero sources | `security_adversarial::empty_aggregate_disputed` | implemented |
| Minority outcome filtered | Outlier dropped; majority retained | `security_adversarial::minority_outcomes_filtered_from_consensus` | implemented |
| Low post-filter agreement | `disputed: true` when kept/total &lt; 0.60 | `security_adversarial::low_agreement_marks_disputed` | implemented |
| Single minority outlier | Outlier dropped, majority wins | `aggregator::outlier_removal_drops_minority` | implemented |
| Tie yes/no weight | `weighted_median` returns `No` (not &gt; 0.5) | `security_adversarial::tie_weight_returns_no` | implemented |
| Unknown-only weights | `weighted_median` → `Unknown` | `security_adversarial::unknown_only_returns_unknown` | implemented |
| Malformed stdin JSON | CLI exits non-zero, no panic | `security_adversarial` + manual CLI | implemented |

## ZK / prover (planned)

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Non-binary outcome witness | Constraints unsatisfied / verify fails | `zk::gadgets::boolean` | planned |
| Wrong public `x_squared` | Verify fails | hello circuit test | planned |
| Tampered proof bytes | Verify fails | `zk_adversarial.rs` | planned |
| Witness ≠ aggregation | Prover errors or verify fails | `from_aggregation` consistency | planned |
| Wrong Poseidon commitment | Verify fails | `gadgets::commitment` | planned |

## CI

All rows marked **implemented** run in `cargo test --workspace` on every PR.
