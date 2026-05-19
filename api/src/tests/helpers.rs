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
}
