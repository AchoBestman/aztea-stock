-- Up migration
ALTER TABLE tenants ADD COLUMN is_system BOOLEAN DEFAULT false NOT NULL;
