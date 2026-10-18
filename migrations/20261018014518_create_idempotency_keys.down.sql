-- migrations/<timestamp>_create_idempotency_keys.down.sql

-- Drop the index and table first
DROP INDEX IF EXISTS idx_idempotency_keys_created_at;
DROP TABLE IF EXISTS idempotency_keys;

-- Drop the custom enum type last
DROP TYPE IF EXISTS idempotency_status;-- Add down migration script here
