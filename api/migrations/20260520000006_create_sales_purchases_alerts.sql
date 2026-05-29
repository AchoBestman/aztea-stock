CREATE TABLE sales (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id         VARCHAR(36) REFERENCES users(id) ON DELETE SET NULL,
    receipt_number  VARCHAR(100) NOT NULL,
    customer_name   VARCHAR(255),
    customer_phone  VARCHAR(50),
    subtotal        DECIMAL(10,2) NOT NULL,
    tax_total       DECIMAL(10,2) DEFAULT 0.0,
    discount_total  DECIMAL(10,2) DEFAULT 0.0,
    total           DECIMAL(10,2) NOT NULL,
    amount_paid     DECIMAL(10,2) NOT NULL,
    change_given    DECIMAL(10,2) DEFAULT 0.0,
    payment_method  VARCHAR(50) NOT NULL CHECK (payment_method IN ('cash','card','mobile_money','credit')),
    status          VARCHAR(50) DEFAULT 'completed' CHECK (status IN ('completed','voided','refunded')),
    notes           TEXT,
    sold_at         DATETIME NOT NULL,
    created_at      DATETIME NOT NULL
);

CREATE TABLE sale_items (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sale_id         VARCHAR(36) NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    product_id      VARCHAR(36) NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    product_name    VARCHAR(255) NOT NULL,
    product_barcode VARCHAR(255),
    quantity        DECIMAL(10,2) NOT NULL,
    unit_price      DECIMAL(10,2) NOT NULL,
    tax_rate        DECIMAL(10,2) DEFAULT 0.0,
    discount        DECIMAL(10,2) DEFAULT 0.0,
    line_total      DECIMAL(10,2) NOT NULL
);

CREATE TABLE purchases (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id         VARCHAR(36) REFERENCES users(id) ON DELETE SET NULL,
    supplier_name   VARCHAR(255),
    supplier_phone  VARCHAR(50),
    reference       VARCHAR(100),
    total           DECIMAL(10,2) NOT NULL,
    status          VARCHAR(50) DEFAULT 'received' CHECK (status IN ('pending','received','partial','cancelled')),
    notes           TEXT,
    purchased_at    DATETIME NOT NULL,
    created_at      DATETIME NOT NULL
);

CREATE TABLE purchase_items (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    purchase_id     VARCHAR(36) NOT NULL REFERENCES purchases(id) ON DELETE CASCADE,
    product_id      VARCHAR(36) NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity        DECIMAL(10,2) NOT NULL,
    unit_cost       DECIMAL(10,2) NOT NULL,
    expiry_date     DATETIME,
    batch_number    VARCHAR(100),
    line_total      DECIMAL(10,2) NOT NULL
);

CREATE TABLE alerts (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id      VARCHAR(36) REFERENCES products(id) ON DELETE CASCADE,
    alert_type      VARCHAR(50) NOT NULL CHECK (alert_type IN ('low_stock','out_of_stock','expiry_soon','expired')),
    message         TEXT NOT NULL,
    threshold       DECIMAL(10,2),
    current_qty     DECIMAL(10,2),
    is_read         BOOLEAN DEFAULT false,
    is_resolved     BOOLEAN DEFAULT false,
    triggered_at    DATETIME NOT NULL
);

CREATE TABLE sync_log (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    device_id       VARCHAR(100) NOT NULL,
    sync_type       VARCHAR(50) CHECK (sync_type IN ('push','pull','full')),
    status          VARCHAR(50) CHECK (status IN ('success','partial','failed')),
    records_pushed  INTEGER DEFAULT 0,
    records_pulled  INTEGER DEFAULT 0,
    error_message   TEXT,
    started_at      DATETIME NOT NULL,
    finished_at     DATETIME
);

CREATE INDEX idx_sales_tenant_date ON sales(tenant_id, sold_at);
CREATE INDEX idx_sales_created ON sales(tenant_id, created_at);
CREATE INDEX idx_sale_items_sale_id ON sale_items(sale_id);
CREATE INDEX idx_purchases_tenant ON purchases(tenant_id);
CREATE INDEX idx_purchase_items_purchase_id ON purchase_items(purchase_id);
CREATE INDEX idx_alerts_tenant_unread ON alerts(tenant_id, is_read);
CREATE INDEX idx_sync_log_tenant ON sync_log(tenant_id);
