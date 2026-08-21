# Adversarial vectors

Attack scenarios mapped to tests. **implemented** = covered in CI today; **manual** = operator audit path without an automated test.

## Fetcher

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Invalid JSON body | `parse_response` returns `InvalidJson` | `security_adversarial::parse_rejects_invalid_json` | implemented |
| Confidence &gt; 1.0 | `InvalidConfidence` | `fetcher::parse_rejects_invalid_confidence` | implemented |
| Confidence &lt; 0.0 | `InvalidConfidence` | `security_adversarial::parse_rejects_negative_confidence` | implemented |
| HTTP 500 / timeout | Source omitted from results | `fetcher::fetch_all_sources_skips_failed` | implemented |
| Body hash at fetch | `raw_hash` is BLAKE3 of HTTP body (F1) | `fetcher` unit tests | implemented |

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

## Storage (M4)

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Proof round-trip | `save_proof` / `get_proof` preserve bytes and public inputs | `storage::save_and_get_proof_round_trip` | implemented |
| Source response archive | `raw_hash` stored per source with `proof_id` | `storage::save_source_responses_links_to_proof` | implemented |
| Prove pipeline persistence | Resolve path writes proof + responses | `storage_prove_pipeline::prove_then_save_and_reload` | implemented |
| Refetch hash audit | Compare stored `raw_hash` to new BLAKE3 of body | operator refetch | manual |

## API (M5)

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Missing API key | 401 on `/resolve` | `api_integration::auth_rejects_missing_key` | implemented |
| Disputed market | 409, no proof stored | `api_integration::resolve_disputed_returns_409` | implemented |
| Rate limit exceeded | 429 | `api_integration::rate_limit_returns_429` | implemented |
| Full resolve pipeline | Proof stored and `/verify` passes | `api_integration::verify_stored_proof_after_resolve` | implemented |

## On-chain (M6)

| Vector | Expected behavior | Test | Status |
| --- | --- | --- | --- |
| Valid mock proof | `resolved(marketId)` true | `PredictionMarketOracleTest::testValidProofAccepts` | implemented |
| Invalid proof | `InvalidProof` revert | `PredictionMarketOracleTest::testInvalidProofReverts` | implemented |
| Double resolve | `AlreadyResolved` revert | `PredictionMarketOracleTest::testDoubleResolveReverts` | implemented |
| Calldata encoding | `public_inputs_u256` matches contract `uint256[2]` | `chain_encoding::mock_proof_matches_public_inputs` | implemented |

## CI

Rows marked **implemented** run in `cargo test --workspace` and `forge test` on every PR.
