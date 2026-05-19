-- Up migration
ALTER TABLE tenants ADD COLUMN is_system BOOLEAN DEFAULT false NOT NULL;
CREATE UNIQUE INDEX uniq_system_tenant ON tenants (is_system) WHERE is_system = true;
