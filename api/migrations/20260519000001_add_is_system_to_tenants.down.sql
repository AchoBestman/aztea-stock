-- Down migration
DROP INDEX IF EXISTS uniq_system_tenant;
ALTER TABLE tenants DROP COLUMN is_system;
