-- Down migration
ALTER TABLE tenants DROP COLUMN two_factor_enabled;
ALTER TABLE users DROP COLUMN two_factor_enabled;
ALTER TABLE users DROP COLUMN two_factor_code;
ALTER TABLE users DROP COLUMN two_factor_expires_at;
