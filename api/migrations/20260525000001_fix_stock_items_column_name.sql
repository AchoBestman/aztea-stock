-- Rename reserved_quantity to quantity_reserved in stock_items for consistency with SeaORM model
-- and fix the column type to match the model's f64 (double)
ALTER TABLE stock_items
    RENAME COLUMN reserved_quantity TO quantity_reserved;