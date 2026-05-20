CREATE TABLE sales (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id         TEXT REFERENCES users(id) ON DELETE SET NULL,
    receipt_number  TEXT NOT NULL,
    customer_name   TEXT,
    customer_phone  TEXT,
    subtotal        REAL NOT NULL,
    tax_total       REAL DEFAULT 0,
    discount_total  REAL DEFAULT 0,
    total           REAL NOT NULL,
    amount_paid     REAL NOT NULL,
    change_given    REAL DEFAULT 0,
    payment_method  TEXT NOT NULL CHECK (payment_method IN ('cash','card','mobile_money','credit')),
    status          TEXT DEFAULT 'completed' CHECK (status IN ('completed','voided','refunded')),
    notes           TEXT,
    sold_at         TEXT NOT NULL,
    created_at      TEXT NOT NULL
);

CREATE TABLE sale_items (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sale_id         TEXT NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    product_name    TEXT NOT NULL,
    product_barcode TEXT,
    quantity        REAL NOT NULL,
    unit_price      REAL NOT NULL,
    tax_rate        REAL DEFAULT 0,
    discount        REAL DEFAULT 0,
    line_total      REAL NOT NULL
);

CREATE TABLE purchases (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id         TEXT REFERENCES users(id) ON DELETE SET NULL,
    supplier_name   TEXT,
    supplier_phone  TEXT,
    reference       TEXT,
    total           REAL NOT NULL,
    status          TEXT DEFAULT 'received' CHECK (status IN ('pending','received','partial','cancelled')),
    notes           TEXT,
    purchased_at    TEXT NOT NULL,
    created_at      TEXT NOT NULL
);

CREATE TABLE purchase_items (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    purchase_id     TEXT NOT NULL REFERENCES purchases(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity        REAL NOT NULL,
    unit_cost       REAL NOT NULL,
    expiry_date     TEXT,
    batch_number    TEXT,
    line_total      REAL NOT NULL
);

CREATE TABLE alerts (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id      TEXT REFERENCES products(id) ON DELETE CASCADE,
    alert_type      TEXT NOT NULL CHECK (alert_type IN ('low_stock','out_of_stock','expiry_soon','expired')),
    message         TEXT NOT NULL,
    threshold       REAL,
    current_qty     REAL,
    is_read         BOOLEAN DEFAULT 0,
    is_resolved     BOOLEAN DEFAULT 0,
    triggered_at    TEXT NOT NULL
);

CREATE TABLE sync_log (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    device_id       TEXT NOT NULL,
    sync_type       TEXT CHECK (sync_type IN ('push','pull','full')),
    status          TEXT CHECK (status IN ('success','partial','failed')),
    records_pushed  INTEGER DEFAULT 0,
    records_pulled  INTEGER DEFAULT 0,
    error_message   TEXT,
    started_at      TEXT NOT NULL,
    finished_at     TEXT
);

CREATE INDEX idx_sales_tenant_date ON sales(tenant_id, sold_at);
CREATE INDEX idx_sales_created ON sales(tenant_id, created_at);
CREATE INDEX idx_sale_items_sale_id ON sale_items(sale_id);
CREATE INDEX idx_purchases_tenant ON purchases(tenant_id);
CREATE INDEX idx_purchase_items_purchase_id ON purchase_items(purchase_id);
CREATE INDEX idx_alerts_tenant_unread ON alerts(tenant_id, is_read);
CREATE INDEX idx_sync_log_tenant ON sync_log(tenant_id);
