-- Up migration
CREATE TABLE subscriptions (
    id VARCHAR(36) PRIMARY KEY,
    tenant_id VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plan VARCHAR(50) NOT NULL CHECK (plan IN ('starter','pro','enterprise')),
    status VARCHAR(50) NOT NULL CHECK (status IN ('trial','active','suspended','cancelled')),
    price_monthly DECIMAL(10,2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'XAF',
    started_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    trial_ends_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    payment_method VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE licenses (
    id VARCHAR(36) PRIMARY KEY,
    tenant_id VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    subscription_id VARCHAR(36) NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    license_key VARCHAR(255) UNIQUE NOT NULL, -- length 255 for AES encryption
    device_name VARCHAR(255),
    device_fingerprint VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    last_verified_at TIMESTAMPTZ,
    activated_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);
