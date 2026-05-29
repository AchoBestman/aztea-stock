-- Remove unique constraints for categories and products names
DROP INDEX idx_categories_tenant_name_active ON categories;
ALTER TABLE categories DROP COLUMN name_active;
DROP INDEX idx_products_tenant_name_active ON products;
ALTER TABLE products DROP COLUMN name_active;