-- Fix price_monthly storage type: DECIMAL → REAL
-- Pour PostgreSQL

-- Créer la nouvelle table avec REAL
CREATE TABLE subscriptions_new (
    id VARCHAR(36) PRIMARY KEY,
    tenant_id VARCHAR(36) NOT NULL,
    plan VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    price_monthly REAL NOT NULL,
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

-- Copier les données
INSERT INTO subscriptions_new
SELECT
    id, tenant_id, plan, status,
    CAST(price_monthly AS REAL),
    currency, max_devices,
    started_at, expires_at,
    trial_ends_at, cancelled_at,
    payment_method, notes, created_at
FROM subscriptions;

-- Remplacer l'ancienne table
DROP TABLE subscriptions CASCADE;
ALTER TABLE subscriptions_new RENAME TO subscriptions