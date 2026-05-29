-- Revert quantity_reserved to reserved_quantity
ALTER TABLE stock_items
    RENAME COLUMN quantity_reserved TO reserved_quantity;