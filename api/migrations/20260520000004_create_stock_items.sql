CREATE TABLE stock_items (
    id                  VARCHAR(36) PRIMARY KEY,
    tenant_id           VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id          VARCHAR(36) NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity            DECIMAL(10,2) NOT NULL DEFAULT 0.0,
    reserved_quantity   DECIMAL(10,2) NOT NULL DEFAULT 0.0,
    unit_cost           DECIMAL(10,2),
    low_stock_threshold REAL NOT NULL DEFAULT 5.0,
    unit_location       VARCHAR(255),
    batch_number        VARCHAR(100),
    expiry_date         DATETIME,
    updated_at          DATETIME NOT NULL,
    CONSTRAINT uniq_tenant_product UNIQUE (tenant_id, product_id)
);

CREATE INDEX idx_stock_items_tenant_id ON stock_items(tenant_id);
CREATE INDEX idx_stock_items_product_id ON stock_items(product_id);
