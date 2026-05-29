CREATE TABLE products (
    id              VARCHAR(36) PRIMARY KEY,
    tenant_id       VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    category_id     VARCHAR(36) REFERENCES categories(id) ON DELETE SET NULL,
    barcode         VARCHAR(255),
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    brand           VARCHAR(100), -- Marque du produit
    unit            VARCHAR(50) NOT NULL DEFAULT 'unité', -- Unité de mesure (ex: kg, boîte, unité)
    purchase_price  REAL NOT NULL DEFAULT 0.0,
    selling_price   REAL NOT NULL DEFAULT 0.0,
    tax_rate        REAL NOT NULL DEFAULT 0.0, -- Taux de taxe/TVA applicable (ex: 18.0)
    image_url       TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    requires_prescription BOOLEAN NOT NULL DEFAULT false,
    created_at      DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL,
    deleted_at      DATETIME,
    CONSTRAINT uniq_tenant_product_barcode UNIQUE (tenant_id, barcode)
);

CREATE INDEX idx_products_tenant_id ON products(tenant_id);
