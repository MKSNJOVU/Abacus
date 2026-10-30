-- Payments, accounts, and journal entries.
-- idempotency_keys is created by the next migration, so this column stores
-- the request UUID without a foreign key.

CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state TEXT NOT NULL DEFAULT 'Initiated',
    amount BIGINT,
    currency TEXT NOT NULL DEFAULT 'USD',
    idempotency_key UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    currency TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id UUID,
    payment_id UUID REFERENCES payments(id),
    account_id UUID REFERENCES accounts(id),
    side TEXT NOT NULL DEFAULT 'Debit',
    amount BIGINT,
    currency TEXT
);

INSERT INTO accounts (name, currency) VALUES ('Settlement Cash', 'USD'), ('Merchant Payable', 'USD');

CREATE INDEX IDX_PaymentId ON journal_entries (payment_id);
CREATE INDEX IDX_AccountId ON journal_entries (account_id);
CREATE INDEX IDX_EntryId ON journal_entries (entry_id);
