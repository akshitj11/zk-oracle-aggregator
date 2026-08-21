-- Indexes for proof lookups and source response joins (M4).

CREATE INDEX IF NOT EXISTS idx_source_responses_proof_id
    ON source_responses (proof_id);

CREATE INDEX IF NOT EXISTS idx_oracle_proofs_created_at
    ON oracle_proofs (created_at DESC);
