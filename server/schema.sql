
-- auth service
-- users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mobile_hash TEXT NOT NULL UNIQUE, -- HMAC of mobile
    device_fingerprint TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- otp_store (ephemeral — 5 min TTL)
CREATE TABLE otp_store (
    mobile_hash TEXT PRIMARY KEY,
    otp TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INT NOT NULL DEFAULT 0
);

-- refresh_tokens (long-lived — 7 days)
CREATE TABLE refresh_tokens (
    token TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    device_fingerprint TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- index for cleanup
CREATE INDEX idx_otp_expired ON otp_store (expires_at) WHERE expires_at < NOW();
CREATE INDEX idx_refresh_expired ON refresh_tokens (expires_at) WHERE expires_at < NOW();
CREATE INDEX idx_refresh_user ON refresh_tokens (user_id);

-- payment-service
-- transaction_journal (immutable, append-only)
CREATE TABLE transaction_journal (
    tx_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id UUID NOT NULL,
    to_user_id UUID NOT NULL,
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL DEFAULT 'INR',
    status TEXT NOT NULL, -- 'SUCCESS', 'FAILED', 'PENDING'
    idempotency_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- daily_limits (track per user per day)
CREATE TABLE daily_limits (
    user_id UUID PRIMARY KEY,
    amount_used BIGINT NOT NULL DEFAULT 0,
    reset_date DATE NOT NULL DEFAULT CURRENT_DATE,
    kyc_tier TEXT NOT NULL DEFAULT 'basic' -- 'basic' or 'full'
);

-- index for cleanup
CREATE INDEX idx_daily_limits_reset ON daily_limits (reset_date);


-- wallet-service
-- wallets table
CREATE TABLE wallets (
    user_id UUID PRIMARY KEY,
    balance BIGINT NOT NULL CHECK (balance >= 0),
    version INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- idempotency keys (24h TTL)
CREATE TABLE idempotency_keys (
    idempotency_key TEXT PRIMARY KEY,
    user_id UUID NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- index for cleanup
CREATE INDEX idx_idempotency_expired ON idempotency_keys (consumed_at)
    WHERE consumed_at < NOW() - INTERVAL '24 hours';

-- audit trigger (optional but recommended)
CREATE TABLE wallet_audit (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    old_balance BIGINT,
    new_balance BIGINT,
    change_amount BIGINT,
    operation TEXT NOT NULL, -- 'CREDIT' or 'DEBIT'
    triggered_by TEXT,       -- 'system', 'user:<id>'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger function
CREATE OR REPLACE FUNCTION log_wallet_change()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO wallet_audit (user_id, old_balance, new_balance, change_amount, operation, triggered_by)
    VALUES (
        NEW.user_id,
        OLD.balance,
        NEW.balance,
        NEW.balance - OLD.balance,
        CASE WHEN NEW.balance > OLD.balance THEN 'CREDIT' ELSE 'DEBIT' END,
        'system'
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger
CREATE TRIGGER wallet_audit_trigger
    AFTER UPDATE ON wallets
    FOR EACH ROW
    WHEN (OLD.balance <> NEW.balance)
    EXECUTE FUNCTION log_wallet_change();



-- fraud_flags table
CREATE TABLE fraud_flags (
    tx_id UUID PRIMARY KEY REFERENCES transaction_journal(tx_id) ON DELETE CASCADE,
    risk_score INT NOT NULL CHECK (risk_score BETWEEN 0 AND 100),
    reason TEXT NOT NULL,
    flagged_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed BOOLEAN NOT NULL DEFAULT FALSE,
    reviewed_at TIMESTAMPTZ
);

-- index for dashboard
CREATE INDEX idx_fraud_unreviewed ON fraud_flags (reviewed) WHERE NOT reviewed;
CREATE INDEX idx_fraud_score ON fraud_flags (risk_score DESC);



-- kyc -provide
-- fake_kyc_verifications
CREATE TABLE fake_kyc_verifications (
    user_id UUID PRIMARY KEY,
    pan TEXT,
    aadhaar TEXT,
    name TEXT NOT NULL,
    dob DATE NOT NULL,
    status TEXT NOT NULL, -- 'pending', 'approved', 'rejected'
    reason TEXT,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- index for status
CREATE INDEX idx_kyc_status ON fake_kyc_verifications (status);




-- fake_bank_accounts
CREATE TABLE fake_bank_accounts (
    user_id UUID PRIMARY KEY,
    account_number TEXT NOT NULL,
    ifsc TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);