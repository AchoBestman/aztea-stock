use sea_orm::{Database, DatabaseConnection, ConnectionTrait, Statement, DatabaseBackend};

pub async fn setup_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    setup_schema(&db).await;
    db
}

pub async fn setup_schema(db: &DatabaseConnection) {
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS tenants (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            business_type VARCHAR(50) NOT NULL CHECK (business_type IN ('pharmacy','supermarket','both')),
            email VARCHAR(255) UNIQUE NOT NULL,
            phone VARCHAR(50),
            address TEXT,
            country VARCHAR(100) DEFAULT 'CG',
            timezone VARCHAR(100) DEFAULT 'Africa/Brazzaville',
            logo_url TEXT,
            is_active BOOLEAN DEFAULT true,
            is_system BOOLEAN DEFAULT false NOT NULL,
            two_factor_enabled BOOLEAN DEFAULT false NOT NULL,
            sender_email TEXT,
            sender_user TEXT,
            sender_password TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS roles (
            id VARCHAR(36) PRIMARY KEY,
            tenant_id VARCHAR(36) NOT NULL,
            name VARCHAR(50) NOT NULL,
            description TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            CONSTRAINT uniq_tenant_role_name UNIQUE (tenant_id, name)
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(36) PRIMARY KEY,
            tenant_id VARCHAR(36) NOT NULL,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            pin_hash VARCHAR(255),
            is_active BOOLEAN DEFAULT true,
            last_login TIMESTAMPTZ,
            two_factor_enabled BOOLEAN DEFAULT false NOT NULL,
            two_factor_code VARCHAR(10),
            two_factor_expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            UNIQUE (tenant_id, email)
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS user_roles (
            user_id VARCHAR(36) NOT NULL,
            role_id VARCHAR(36) NOT NULL,
            PRIMARY KEY (user_id, role_id),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS permissions (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) UNIQUE NOT NULL,
            description TEXT,
            model_group VARCHAR(50) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS role_permissions (
            role_id VARCHAR(36) NOT NULL,
            permission_id VARCHAR(36) NOT NULL,
            PRIMARY KEY (role_id, permission_id),
            FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
            FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            color TEXT,
            icon TEXT,
            parent_id TEXT,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            category_id TEXT,
            barcode TEXT,
            name TEXT NOT NULL,
            description TEXT,
            brand TEXT,
            unit TEXT NOT NULL DEFAULT 'unité',
            purchase_price REAL NOT NULL DEFAULT 0.0,
            selling_price REAL NOT NULL DEFAULT 0.0,
            tax_rate REAL NOT NULL DEFAULT 0.0,
            image_url TEXT,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            requires_prescription BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
            UNIQUE(tenant_id, barcode)
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS stock_items (
            id                  TEXT PRIMARY KEY,
            tenant_id           TEXT NOT NULL,
            product_id          TEXT NOT NULL,
            quantity            REAL NOT NULL DEFAULT 0.0,
            quantity_reserved   REAL NOT NULL DEFAULT 0.0,
            low_stock_threshold REAL NOT NULL DEFAULT 5.0,
            unit_location       TEXT,
            batch_number        TEXT,
            expiry_date         TEXT,
            updated_at          TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
            UNIQUE (tenant_id, product_id)
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS stock_movements (
            id              TEXT PRIMARY KEY,
            tenant_id       TEXT NOT NULL,
            product_id      TEXT NOT NULL,
            user_id         TEXT,
            movement_type   TEXT NOT NULL,
            quantity_before REAL NOT NULL,
            quantity_change REAL NOT NULL,
            quantity_after  REAL NOT NULL,
            reference_id    TEXT,
            note            TEXT,
            occurred_at     TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS sales (
            id              TEXT PRIMARY KEY,
            tenant_id       TEXT NOT NULL,
            user_id         TEXT,
            receipt_number  TEXT NOT NULL,
            customer_name   TEXT,
            customer_phone  TEXT,
            subtotal        REAL NOT NULL,
            tax_total       REAL DEFAULT 0,
            discount_total  REAL DEFAULT 0,
            total           REAL NOT NULL,
            amount_paid     REAL NOT NULL,
            change_given    REAL DEFAULT 0,
            payment_method  TEXT NOT NULL,
            status          TEXT DEFAULT 'completed',
            notes           TEXT,
            sold_at         TEXT NOT NULL,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS sale_items (
            id              TEXT PRIMARY KEY,
            tenant_id       TEXT NOT NULL,
            sale_id         TEXT NOT NULL,
            product_id      TEXT NOT NULL,
            product_name    TEXT NOT NULL,
            product_barcode TEXT,
            quantity        REAL NOT NULL,
            unit_price      REAL NOT NULL,
            tax_rate        REAL DEFAULT 0,
            discount        REAL DEFAULT 0,
            line_total      REAL NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (sale_id) REFERENCES sales(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS purchases (
            id              TEXT PRIMARY KEY,
            tenant_id       TEXT NOT NULL,
            user_id         TEXT,
            supplier_name   TEXT,
            supplier_phone  TEXT,
            reference       TEXT,
            total           REAL NOT NULL,
            status          TEXT DEFAULT 'received',
            notes           TEXT,
            purchased_at    TEXT NOT NULL,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS purchase_items (
            id              TEXT PRIMARY KEY,
            tenant_id       TEXT NOT NULL,
            purchase_id     TEXT NOT NULL,
            product_id      TEXT NOT NULL,
            quantity        REAL NOT NULL,
            unit_cost       REAL NOT NULL,
            expiry_date     TEXT,
            batch_number    TEXT,
            line_total      REAL NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (purchase_id) REFERENCES purchases(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS alerts (
            id              TEXT PRIMARY KEY,
            tenant_id       TEXT NOT NULL,
            product_id      TEXT,
            alert_type      TEXT NOT NULL,
            message         TEXT NOT NULL,
            threshold       REAL,
            current_qty     REAL,
            is_read         BOOLEAN DEFAULT 0,
            is_resolved     BOOLEAN DEFAULT 0,
            triggered_at    TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, "
        CREATE TABLE IF NOT EXISTS sync_log (
            id              TEXT PRIMARY KEY,
            tenant_id       TEXT NOT NULL,
            device_id       TEXT NOT NULL,
            sync_type       TEXT,
            status          TEXT,
            records_pushed  INTEGER DEFAULT 0,
            records_pulled  INTEGER DEFAULT 0,
            error_message   TEXT,
            started_at      TEXT NOT NULL,
            finished_at     TEXT,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
        );
    ".to_string())).await.unwrap();
}
