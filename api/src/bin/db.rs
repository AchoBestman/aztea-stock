use std::env;
use std::fs;
use sqlx::{AnyPool, Row};
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};

#[path = "../config.rs"]
mod config;
async fn create_pool(config: &config::Config) -> Option<AnyPool> {
    let url = if config.offline || config.db_type == "sqlite" {
        &config.sqlite_database_url
    } else {
        match &config.database_url {
            Some(u) => u,
            None => &config.sqlite_database_url
        }
    };
    sqlx::any::install_default_drivers();
    sqlx::any::AnyPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(url)
        .await
        .ok()
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    // Initialize logging so we see connection pool logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = args[1].as_str();

    let config = config::Config::from_env()?;
    let pool = match create_pool(&config).await {
        Some(p) => p,
        None => {
            eprintln!("Error: Failed to connect to the database.");
            std::process::exit(1);
        }
    };

    match command {
        "migrate" => {
            run_migrations(&pool).await?;
        }
        "rollback" => {
            run_rollback(&pool).await?;
        }
        "seed" => {
            run_seeds(&pool).await?;
        }
        "fresh" => {
            run_fresh(&pool, &config).await?;
        }
        "setup" => {
            println!("Starting database setup...");
            run_fresh(&pool, &config).await?;
            run_migrations(&pool).await?;
            run_seeds(&pool).await?;
            println!("Database setup completed successfully!");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage: cargo run --bin db <command>");
    println!("\nAvailable commands:");
    println!("  migrate  - Run all database migrations");
    println!("  rollback - Rollback the last migration");
    println!("  seed     - Seed default tenant, roles, permissions, and Super Admin");
    println!("  fresh    - Clear database schema (drops all tables)");
    println!("  setup    - Complete reset: fresh + migrate + seed");
}

async fn run_migrations(pool: &AnyPool) -> Result<(), anyhow::Error> {
    println!("Running database migrations...");
    sqlx::migrate!("./migrations").run(pool).await?;
    println!("Migrations executed successfully.");
    Ok(())
}

async fn run_rollback(pool: &AnyPool) -> Result<(), anyhow::Error> {
    println!("Undoing the last migration...");
    
    // Check if the migrations table exists
    // We check sqlite_master first. If that query fails (which happens on Postgres), we query pg_tables.
    let table_exists: bool = match sqlx::query(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'"
    ).fetch_one(pool).await {
        Ok(row) => {
            let count: i32 = row.try_get(0)?;
            count > 0
        }
        Err(_) => {
            let row = sqlx::query(
                "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = '_sqlx_migrations')"
            ).fetch_one(pool).await?;
            row.try_get(0)?
        }
    };
    
    if !table_exists {
        println!("No migrations table found. Nothing to rollback.");
        return Ok(());
    }
    
    // Get the last applied migration
    let last_migration = sqlx::query("SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1")
        .fetch_optional(pool)
        .await?;
        
    let row = match last_migration {
        Some(row) => row,
        None => {
            println!("No migrations found to rollback.");
            return Ok(());
        }
    };
    
    let version: i64 = row.try_get(0)?;
    
    // Search the migrations directory for the corresponding .down.sql file
    let paths = fs::read_dir("./migrations")?;
    let mut down_sql_file = None;
    for path in paths {
        let path = path?.path();
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            if file_name.starts_with(&version.to_string()) && file_name.ends_with(".down.sql") {
                down_sql_file = Some(path);
                break;
            }
        }
    }
    
    if let Some(file_path) = down_sql_file {
        println!("Running down migration file: {:?}", file_path);
        let sql = fs::read_to_string(file_path)?;
        
        let mut tx = pool.begin().await?;
        sqlx::query(&sql).execute(&mut *tx).await?;
        
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
            .bind(version)
            .execute(&mut *tx)
            .await?;
            
        tx.commit().await?;
        println!("Migration {} successfully rolled back.", version);
    } else {
        println!("No .down.sql file found for version {}. Manual rollback required.", version);
    }
    
    Ok(())
}

async fn run_fresh(pool: &AnyPool, config: &config::Config) -> Result<(), anyhow::Error> {
    let is_sqlite = config.offline || config.db_type == "sqlite" || config.database_url.is_none();
    
    if !is_sqlite {
        println!("Dropping and recreating PostgreSQL public schema...");
        sqlx::query("DROP SCHEMA public CASCADE").execute(pool).await?;
        sqlx::query("CREATE SCHEMA public").execute(pool).await?;
        sqlx::query("GRANT ALL ON SCHEMA public TO public").execute(pool).await?;
    } else {
        println!("Dropping SQLite database tables...");
        // Disable foreign keys checks
        sqlx::query("PRAGMA foreign_keys = OFF;").execute(pool).await?;
        
        let tables_to_drop = vec![
            "role_permissions",
            "user_roles",
            "users",
            "roles",
            "permissions",
            "tenants",
            "_sqlx_migrations",
        ];
        
        for table in tables_to_drop {
            sqlx::query(&format!("DROP TABLE IF EXISTS {}", table)).execute(pool).await?;
        }
        
        sqlx::query("PRAGMA foreign_keys = ON;").execute(pool).await?;
    }
    
    println!("Database cleared successfully.");
    Ok(())
}

async fn run_seeds(pool: &AnyPool) -> Result<(), anyhow::Error> {
    println!("Seeding database...");

    // 1. Create System Tenant
    let tenant_id = Uuid::new_v4().to_string();
    let tenant_name = env::var("SYSTEM_TENANT_NAME")
        .unwrap_or_else(|_| "Aztea Software (Système)".to_string());
    let tenant_business_type = env::var("SYSTEM_TENANT_BUSINESS_TYPE")
        .unwrap_or_else(|_| "both".to_string());
    let tenant_email = env::var("SYSTEM_TENANT_EMAIL")
        .unwrap_or_else(|_| "contact@aztea.com".to_string());
    let tenant_phone = env::var("SYSTEM_TENANT_PHONE").ok();
    let tenant_address = env::var("SYSTEM_TENANT_ADDRESS").ok();

    sqlx::query(
        "INSERT INTO tenants (id, name, business_type, email, phone, address, is_system) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&tenant_id)
    .bind(&tenant_name)
    .bind(&tenant_business_type)
    .bind(&tenant_email)
    .bind(tenant_phone)
    .bind(tenant_address)
    .bind(true)
    .execute(pool)
    .await?;
    println!("Created system tenant: {}", tenant_name);

    // 2. Create Permissions
    let permissions_data = vec![
        // Roles
        ("can_create_role", "Permet de créer des rôles", "roles"),
        ("can_read_role", "Permet de lire les rôles", "roles"),
        ("can_update_role", "Permet de modifier les rôles", "roles"),
        ("can_delete_role", "Permet de supprimer les rôles", "roles"),
        ("can_assign_role_to_user", "Permet d'assigner des rôles aux utilisateurs", "roles"),
        // Products
        ("can_create_product", "Permet de créer des produits", "products"),
        ("can_read_product", "Permet de lire les produits", "products"),
        ("can_update_product", "Permet de modifier les produits", "products"),
        ("can_delete_product", "Permet de supprimer les produits", "products"),
        // Sales
        ("can_create_sale", "Permet d'enregistrer des ventes", "sales"),
        ("can_read_sale", "Permet de lire les ventes", "sales"),
        ("can_update_sale", "Permet de modifier les ventes", "sales"),
        ("can_delete_sale", "Permet de supprimer les ventes", "sales"),
        // Tenants
        ("can_create_tenant", "Permet de créer des tenants", "tenants"),
        ("can_read_tenant", "Permet de lire les tenants", "tenants"),
        ("can_update_tenant", "Permet de modifier les tenants", "tenants"),
        ("can_delete_tenant", "Permet de supprimer les tenants", "tenants"),
    ];

    let mut permission_ids = Vec::new();

    for (name, desc, group) in permissions_data {
        let perm_id = Uuid::new_v4().to_string();
        
        let row = sqlx::query(
            "INSERT INTO permissions (id, name, description, model_group) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&perm_id)
        .bind(name)
        .bind(desc)
        .bind(group)
        .fetch_one(pool)
        .await?;
        
        let id: String = row.try_get(0)?;
        permission_ids.push(id);
    }
    println!("Seeded {} permissions.", permission_ids.len());

    // 3. Create Super Admin Role for the system tenant
    let super_admin_role_id = Uuid::new_v4().to_string();
    let role_name = "Super Admin";
    
    let role_row = sqlx::query(
        "INSERT INTO roles (id, tenant_id, name, description) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&super_admin_role_id)
    .bind(&tenant_id)
    .bind(role_name)
    .bind("Administrateur suprême du système avec tous les accès")
    .fetch_one(pool)
    .await?;
    let actual_role_id: String = role_row.try_get(0)?;
    println!("Created role: {}", role_name);

    // 4. Assign all permissions to the Super Admin Role
    for perm_id in &permission_ids {
        sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)")
            .bind(&actual_role_id)
            .bind(perm_id)
            .execute(pool)
            .await?;
    }
    println!("Assigned all permissions to Super Admin role.");

    // 5. Create default Roles for system tenant (Admin, Manager, User)
    let default_roles = vec![
        ("Admin", "Administrateur de tenant"),
        ("Manager", "Gestionnaire de stock et ventes"),
        ("User", "Utilisateur standard"),
    ];
    for (name, desc) in default_roles {
        let id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO roles (id, tenant_id, name, description) VALUES ($1, $2, $3, $4)")
            .bind(&id)
            .bind(&tenant_id)
            .bind(name)
            .bind(desc)
            .execute(pool)
            .await?;
    }
    println!("Seeded default roles (Admin, Manager, User) for system tenant.");

    // 6. Create Super Admin User
    let sa_email = env::var("SUPER_ADMIN_EMAIL").unwrap_or_else(|_| "superadmin@aztea.com".to_string());
    let sa_password = env::var("SUPER_ADMIN_PASSWORD").unwrap_or_else(|_| "SuperSecurePassword123!".to_string());
    let password_hash = hash(&sa_password, DEFAULT_COST)?;

    let user_id = Uuid::new_v4().to_string();
    let user_row = sqlx::query(
        "INSERT INTO users (id, tenant_id, name, email, password_hash) VALUES ($1, $2, $3, $4, $5) RETURNING id"
    )
    .bind(&user_id)
    .bind(&tenant_id)
    .bind("Super Administrateur")
    .bind(&sa_email)
    .bind(&password_hash)
    .fetch_one(pool)
    .await?;
    let actual_user_id: String = user_row.try_get(0)?;
    println!("Created Super Admin User with email: {}", sa_email);

    // 7. Link Super Admin User to Super Admin Role
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
        .bind(&actual_user_id)
        .bind(&actual_role_id)
        .execute(pool)
        .await?;
    println!("Assigned Super Admin role to user: {}", sa_email);

    println!("Seeding completed successfully!");
    Ok(())
}
