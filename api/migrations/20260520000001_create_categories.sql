CREATE TABLE categories (
    id          VARCHAR(36) PRIMARY KEY,
    tenant_id   VARCHAR(36) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    color       VARCHAR(50),             -- hex color pour l'UI
    icon        VARCHAR(100),
    parent_id   VARCHAR(36) REFERENCES categories(id) ON DELETE SET NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at  DATETIME              -- soft delete
);

CREATE INDEX idx_categories_tenant_id ON categories(tenant_id);
