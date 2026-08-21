# Adversarial vectors

Attack scenarios mapped to tests. **implemented** = covered in CI today; **planned** = future milestone.

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

## ZK / prover

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Non-binary outcome witness | Circuit unsatisfied | `zk_invariants::z1_outcome_witnesses_are_binary` | implemented |
| Excluded source zero weight | Outlier does not count in-circuit | `zk_invariants::z2_excluded_sources_do_not_count_toward_total` | implemented |
| Public source_count mismatch | Verify fails | `prover_roundtrip::wrong_source_count_public_input_fails` | implemented |
| Witness matches aggregate | Honest witness equals M2 output | `zk_invariants::z6_witness_matches_aggregate` | implemented |
| Disputed market witness | `build_witness` errors | `zk_invariants::z6_rejects_disputed_witness` | implemented |
| Tampered proof bytes | Verify fails | `prover_roundtrip::tampered_proof_bytes_fail_verify` | implemented |
| Tampered public inputs | Verify fails | `prover_roundtrip::tampered_public_inputs_fail_verify` | implemented |
| Groth16 round-trip | Valid proof verifies | `zk_invariants::z7_z8_groth16_verify_invariants` | implemented |
| Body hash mismatch (audit) | Off-chain compare `raw_hash` vs BLAKE3 | M4 storage | planned |

## CI

All rows marked **implemented** run in `cargo test --workspace` on every PR.

## API (M5)

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Missing API key | 401 on `/resolve` | `api_integration::auth_rejects_missing_key` | implemented |
| Disputed market | 409, no proof stored | `api_integration::resolve_disputed_returns_409` | implemented |
| Rate limit exceeded | 429 | `api_integration::rate_limit_returns_429` | implemented |
