CREATE TABLE categories (
    id          TEXT PRIMARY KEY,
    tenant_id   TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    description TEXT,
    color       TEXT,             -- hex color pour l'UI
    icon        TEXT,
    parent_id   TEXT REFERENCES categories(id) ON DELETE SET NULL,
    created_at  TEXT NOT NULL,    -- TIMESTAMPTZ stored as text in sqlite
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT              -- soft delete
);

CREATE INDEX idx_categories_tenant_id ON categories(tenant_id);
