CREATE TABLE stock_items (
    id                  TEXT PRIMARY KEY,
    tenant_id           TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id          TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity            REAL NOT NULL DEFAULT 0.0,
    quantity_reserved   REAL NOT NULL DEFAULT 0.0,
    low_stock_threshold REAL NOT NULL DEFAULT 5.0,
    unit_location       TEXT,
    batch_number        TEXT,
    expiry_date         TEXT,
    updated_at          TEXT NOT NULL,
    CONSTRAINT uniq_tenant_product UNIQUE (tenant_id, product_id)
);

CREATE INDEX idx_stock_items_tenant_id ON stock_items(tenant_id);
CREATE INDEX idx_stock_items_product_id ON stock_items(product_id);
