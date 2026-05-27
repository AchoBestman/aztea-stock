CREATE TABLE stock_movements (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id      VARCHAR(36) NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    user_id         VARCHAR(36) REFERENCES users(id) ON DELETE SET NULL,
    movement_type   VARCHAR(50) NOT NULL CHECK (movement_type IN ('sale','purchase','adjustment','return','loss','initial')),
    quantity_before DECIMAL(10,2) NOT NULL,
    quantity_change DECIMAL(10,2) NOT NULL,
    quantity_after  DECIMAL(10,2) NOT NULL,
    reference_id    VARCHAR(100),
    note            TEXT,
    occurred_at     TIMESTAMP NOT NULL
);

CREATE INDEX idx_stock_movements_tenant_id ON stock_movements(tenant_id);
CREATE INDEX idx_stock_movements_product_id ON stock_movements(product_id);
