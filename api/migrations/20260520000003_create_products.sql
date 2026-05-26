CREATE TABLE products (
    id              TEXT PRIMARY KEY,
    tenant_id       TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    category_id     TEXT REFERENCES categories(id) ON DELETE SET NULL,
    barcode         TEXT,
    name            TEXT NOT NULL,
    description     TEXT,
    brand           TEXT, -- Marque du produit
    unit            TEXT NOT NULL DEFAULT 'unité', -- Unité de mesure (ex: kg, boîte, unité)
    purchase_price  REAL NOT NULL DEFAULT 0.0,
    selling_price   REAL NOT NULL DEFAULT 0.0,
    tax_rate        REAL NOT NULL DEFAULT 0.0, -- Taux de taxe/TVA applicable (ex: 18.0)
    image_url       TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    requires_prescription BOOLEAN NOT NULL DEFAULT false,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT,
    CONSTRAINT uniq_tenant_product_barcode UNIQUE (tenant_id, barcode)
);

CREATE INDEX idx_products_tenant_id ON products(tenant_id);
