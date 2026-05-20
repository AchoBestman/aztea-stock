CREATE TABLE stock_movements (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    user_id         TEXT REFERENCES users(id) ON DELETE SET NULL,
    movement_type   TEXT NOT NULL CHECK (movement_type IN ('sale','purchase','adjustment','return','loss','initial')),
    quantity_before REAL NOT NULL,
    quantity_change REAL NOT NULL,
    quantity_after  REAL NOT NULL,
    reference_id    TEXT,
    note            TEXT,
    occurred_at     TEXT NOT NULL
);

CREATE INDEX idx_stock_movements_tenant_id ON stock_movements(tenant_id);
CREATE INDEX idx_stock_movements_product_id ON stock_movements(product_id);
