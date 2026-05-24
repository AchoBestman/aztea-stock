-- Revert: restore original DECIMAL type (no data loss, just affinity change)
CREATE TABLE subscriptions_old (
    id VARCHAR(36) PRIMARY KEY,
    tenant_id VARCHAR(36) NOT NULL,
    plan VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    price_monthly DECIMAL(10,2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'XAF',
    max_devices INTEGER NOT NULL DEFAULT 1,
    started_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    trial_ends_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    payment_method VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

INSERT INTO subscriptions_old SELECT * FROM subscriptions;
DROP TABLE subscriptions;
ALTER TABLE subscriptions_old RENAME TO subscriptions;
