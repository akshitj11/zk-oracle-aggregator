CREATE TABLE IF NOT EXISTS oracle_proofs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id       BYTEA NOT NULL UNIQUE,
    outcome         BOOLEAN NOT NULL,
    source_count    INTEGER NOT NULL,
    agreement_ratio NUMERIC(5, 4) NOT NULL,
    confidence      NUMERIC(5, 4) NOT NULL,
    proof_bytes     BYTEA NOT NULL,
    public_inputs   JSONB NOT NULL,
    onchain_tx_hash BYTEA,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS source_responses (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proof_id    UUID REFERENCES oracle_proofs (id),
    source_id   TEXT NOT NULL,
    outcome     TEXT NOT NULL,
    confidence  NUMERIC(5, 4),
    raw_hash    BYTEA NOT NULL,
    fetched_at  TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS source_reputation (
    source_id       TEXT PRIMARY KEY,
    correct_count   INTEGER DEFAULT 0,
    total_count     INTEGER DEFAULT 0,
    current_weight  NUMERIC(5, 4) DEFAULT 0.5,
    last_updated    TIMESTAMPTZ DEFAULT NOW()
);
