-- Add unique constraints for categories and products names per tenant (active records only)
-- MySQL doesn't support partial indexes (WHERE), so we use a virtual column approach
ALTER TABLE categories
ADD COLUMN name_active VARCHAR(255) AS (IF(deleted_at IS NULL, name, NULL)) VIRTUAL;
CREATE UNIQUE INDEX idx_categories_tenant_name_active ON categories(tenant_id, name_active);
ALTER TABLE products
ADD COLUMN name_active VARCHAR(255) AS (IF(deleted_at IS NULL, name, NULL)) VIRTUAL;
CREATE UNIQUE INDEX idx_products_tenant_name_active ON products(tenant_id, name_active);