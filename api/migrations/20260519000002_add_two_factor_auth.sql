-- Up migration
ALTER TABLE tenants ADD COLUMN two_factor_enabled BOOLEAN DEFAULT false NOT NULL;
ALTER TABLE users ADD COLUMN two_factor_enabled BOOLEAN DEFAULT false NOT NULL;
ALTER TABLE users ADD COLUMN two_factor_code VARCHAR(10);
ALTER TABLE users ADD COLUMN two_factor_expires_at DATETIME;
