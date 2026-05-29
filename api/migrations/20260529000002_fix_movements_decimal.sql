-- Fix type mismatch for stock_movements columns
ALTER TABLE stock_movements
MODIFY COLUMN quantity_before DOUBLE NOT NULL,
    MODIFY COLUMN quantity_change DOUBLE NOT NULL,
    MODIFY COLUMN quantity_after DOUBLE NOT NULL;