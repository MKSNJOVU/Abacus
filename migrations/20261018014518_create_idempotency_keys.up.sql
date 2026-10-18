-- migrations/<timestamp>_create_idempotency_keys.up.sql

-- 1. Create the custom enum type for tracking request lifecycle state
CREATE TYPE idempotency_status AS ENUM (
    'IN_FLIGHT',
    'COMPLETED',
    'FAILED'
);

-- 2. Create the idempotency table to store request locks and cached responses
CREATE TABLE idempotency_keys (
                                  key UUID PRIMARY KEY,
                                  payload_hash  TEXT NOT NULL,
                                  status        idempotency_status NOT NULL DEFAULT 'IN_FLIGHT',
                                  status_code   SMALLINT,
                                  response_body TEXT,
                                  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Index on created_at so background cleanup/self-healing jobs can efficiently expire old keys
CREATE INDEX idx_idempotency_keys_created_at ON idempotency_keys (created_at);-- Add up migration script here
