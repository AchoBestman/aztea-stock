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
    // Embed and run migrations (force recompile)
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
            "licenses",
            "subscriptions",
            "sync_log",
            "alerts",
            "purchase_items",
            "purchases",
            "sale_items",
            "sales",
            "stock_movements",
            "stock_items",
            "products",
            "categories",
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

    // 1. Create/Retrieve System Tenant
    let tenant_email = env::var("SYSTEM_TENANT_EMAIL")
        .unwrap_or_else(|_| "contact@aztea.com".to_string());

    let existing_tenant = sqlx::query("SELECT id FROM tenants WHERE email = $1")
        .bind(&tenant_email)
        .fetch_optional(pool)
        .await?;

    let tenant_id = match existing_tenant {
        Some(row) => {
            let id: String = row.try_get(0)?;
            println!("System tenant already exists (ID: {}). Skipping insertion.", id);
            id
        }
        None => {
            let new_id = Uuid::new_v4().to_string();
            let tenant_name = env::var("SYSTEM_TENANT_NAME")
                .unwrap_or_else(|_| "Aztea Software (Système)".to_string());
            let tenant_business_type = env::var("SYSTEM_TENANT_BUSINESS_TYPE")
                .unwrap_or_else(|_| "both".to_string());
            let tenant_phone = env::var("SYSTEM_TENANT_PHONE").ok();
            let tenant_address = env::var("SYSTEM_TENANT_ADDRESS").ok();

            sqlx::query(
                "INSERT INTO tenants (id, name, business_type, email, phone, address, is_system, two_factor_enabled) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&new_id)
            .bind(&tenant_name)
            .bind(&tenant_business_type)
            .bind(&tenant_email)
            .bind(tenant_phone)
            .bind(tenant_address)
            .bind(true)
            .bind(false)
            .execute(pool)
            .await?;
            println!("Created system tenant: {}", tenant_name);
            new_id
        }
    };

    // 2. Create Permissions
    let permissions_data = vec![
        // Roles
        ("can_create_role", "Permet de créer des rôles", "roles"),
        ("can_read_role", "Permet de lire les rôles", "roles"),
        ("can_update_role", "Permet de modifier les rôles", "roles"),
        ("can_delete_role", "Permet de supprimer les rôles", "roles"),
        ("can_assign_role_to_user", "Permet d'assigner des rôles aux utilisateurs", "roles"),
        ("can_read_permission", "Permet de voir la liste des permissions système", "roles"),
        // Categories
        ("can_create_category", "Permet de créer des catégories de produits", "categories"),
        ("can_read_category", "Permet de lire les catégories de produits", "categories"),
        ("can_update_category", "Permet de modifier les catégories de produits", "categories"),
        ("can_delete_category", "Permet de supprimer les catégories de produits", "categories"),
        // Products
        ("can_create_product", "Permet de créer des produits", "products"),
        ("can_read_product", "Permet de lire les produits", "products"),
        ("can_update_product", "Permet de modifier les produits", "products"),
        ("can_delete_product", "Permet de supprimer les produits", "products"),
        // Stock
        ("can_read_stock", "Permet de lire les fiches stock et les mouvements", "stock"),
        ("can_manage_stock", "Permet de créer et modifier les fiches stock et d'enregistrer des mouvements", "stock"),
        // Sales
        ("can_create_sale", "Permet d'enregistrer des ventes", "sales"),
        ("can_read_sale", "Permet de lire les ventes", "sales"),
        ("can_update_sale", "Permet de modifier les ventes", "sales"),
        ("can_delete_sale", "Permet de supprimer les ventes", "sales"),
        ("can_export_sale_pdf", "Permet d'exporter l'historique des ventes en PDF", "sales"),
        ("can_export_sale_excel", "Permet d'exporter l'historique des ventes en Excel/CSV", "sales"),
        ("can_print_sale_receipt", "Permet d'imprimer un reçu de vente", "sales"),
        // Purchases
        ("can_create_purchase", "Permet de créer des achats", "purchases"),
        ("can_read_purchase", "Permet de lire les achats", "purchases"),
        ("can_update_purchase", "Permet de modifier les achats", "purchases"),
        ("can_delete_purchase", "Permet de supprimer les achats", "purchases"),
        // Alerts
        ("can_read_alert", "Permet de lire les alertes", "alerts"),
        ("can_manage_alert", "Permet de gérer/lire les alertes", "alerts"),
        // Sync
        ("can_read_sync_log", "Permet de lire le journal de sync", "sync"),
        ("can_manage_sync_log", "Permet de gérer le journal de sync", "sync"),
        // Auth / Appareils
        ("can_read_device_key", "Permet d'obtenir la clé de chiffrement de l'appareil", "auth"),
        // Tenants
        ("can_create_tenant", "Permet de créer des tenants", "tenants"),
        ("can_read_tenant", "Permet de lire les tenants", "tenants"),
        ("can_update_tenant", "Permet de modifier les tenants", "tenants"),
        ("can_delete_tenant", "Permet de supprimer les tenants", "tenants"),
        ("can_set_tenant_two_factor", "Permet de configurer le Two Factor d'un tenant", "tenants"),
        ("can_update_tenant_credentials", "Permet de modifier les identifiants SMTP de connexion d'un tenant", "tenants"),
        // Cross-Tenant (System Only)
        ("can_access_other_tenant_for_edition", "Permet de lire les données des autres tenants", "cross-tenant"),
        ("can_access_other_tenant_for_creation", "Permet de créer des données pour les autres tenants", "cross-tenant"),
        ("can_access_other_tenant_for_updating", "Permet de modifier les données des autres tenants", "cross-tenant"),
        ("can_access_other_tenant_for_deleting", "Permet de supprimer les données des autres tenants", "cross-tenant"),
        // Users
        ("can_create_user", "Permet d'ajouter un utilisateur", "users"),
        ("can_read_user", "Permet de voir les utilisateurs", "users"),
        ("can_update_user", "Permet de modifier un utilisateur", "users"),
        ("can_delete_user", "Permet de supprimer un utilisateur", "users"),
        ("can_manage_tenant_users", "Permet de gérer les comptes utilisateurs d'un tenant", "users"),
        ("can_send_tenant_password_reset", "Permet d'envoyer un email de réinitialisation de mot de passe", "users"),
        ("can_set_tenant_password", "Permet de définir le mot de passe d'un utilisateur", "users"),
        ("can_update_user_status", "Permet de changer le statut d'un utilisateur", "users"),
        ("can_update_user_two_factor", "Permet de configurer la double authentification pour un utilisateur", "users"),
    ];

    let mut permission_ids = Vec::new();

    for (name, desc, group) in permissions_data {
        let existing_perm = sqlx::query("SELECT id FROM permissions WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await?;

        let perm_id = match existing_perm {
            Some(row) => {
                row.try_get::<String, _>(0)?
            }
            None => {
                let new_id = Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO permissions (id, name, description, model_group) VALUES ($1, $2, $3, $4)"
                )
                .bind(&new_id)
                .bind(name)
                .bind(desc)
                .bind(group)
                .execute(pool)
                .await?;
                new_id
            }
        };
        permission_ids.push(perm_id);
    }
    println!("Seeded {} permissions.", permission_ids.len());

    // 3. Create Super Admin Role for the system tenant
    let role_name = "Super Admin";
    let existing_role = sqlx::query("SELECT id FROM roles WHERE tenant_id = $1 AND name = $2")
        .bind(&tenant_id)
        .bind(role_name)
        .fetch_optional(pool)
        .await?;

    let super_admin_role_id = match existing_role {
        Some(row) => {
            let id: String = row.try_get(0)?;
            println!("Role '{}' already exists (ID: {}). Skipping insertion.", role_name, id);
            id
        }
        None => {
            let new_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO roles (id, tenant_id, name, description) VALUES ($1, $2, $3, $4)"
            )
            .bind(&new_id)
            .bind(&tenant_id)
            .bind(role_name)
            .bind("Administrateur suprême du système avec tous les accès")
            .execute(pool)
            .await?;
            println!("Created role: {}", role_name);
            new_id
        }
    };

    // 4. Assign all permissions to the Super Admin Role
    let mut assigned_count = 0;
    for perm_id in &permission_ids {
        let existing_assignment = sqlx::query("SELECT 1 FROM role_permissions WHERE role_id = $1 AND permission_id = $2")
            .bind(&super_admin_role_id)
            .bind(perm_id)
            .fetch_optional(pool)
            .await?;

        if existing_assignment.is_none() {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)")
                .bind(&super_admin_role_id)
                .bind(perm_id)
                .execute(pool)
                .await?;
            assigned_count += 1;
        }
    }
    if assigned_count > 0 {
        println!("Assigned {} new permissions to Super Admin role.", assigned_count);
    } else {
        println!("All permissions already assigned to Super Admin role.");
    }

    // 5. Create default Roles for system tenant (Admin, Manager, User)
    let default_roles = vec![
        ("Admin", "Administrateur de tenant"),
        ("Manager", "Gestionnaire de stock et ventes"),
        ("User", "Utilisateur standard"),
    ];
    for (name, desc) in default_roles {
        let existing_default_role = sqlx::query("SELECT 1 FROM roles WHERE tenant_id = $1 AND name = $2")
            .bind(&tenant_id)
            .bind(name)
            .fetch_optional(pool)
            .await?;

        if existing_default_role.is_none() {
            let new_id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO roles (id, tenant_id, name, description) VALUES ($1, $2, $3, $4)")
                .bind(&new_id)
                .bind(&tenant_id)
                .bind(name)
                .bind(desc)
                .execute(pool)
                .await?;
            println!("Created default role: {}", name);
        }
    }

    // 6. Create Super Admin User
    let sa_email = env::var("SUPER_ADMIN_EMAIL").unwrap_or_else(|_| "superadmin@aztea.com".to_string());
    let existing_user = sqlx::query("SELECT id FROM users WHERE tenant_id = $1 AND email = $2")
        .bind(&tenant_id)
        .bind(&sa_email)
        .fetch_optional(pool)
        .await?;

    let super_admin_user_id = match existing_user {
        Some(row) => {
            let id: String = row.try_get(0)?;
            println!("Super Admin User already exists (ID: {}). Skipping insertion.", id);
            id
        }
        None => {
            let sa_password = env::var("SUPER_ADMIN_PASSWORD").unwrap_or_else(|_| "SuperSecurePassword123!".to_string());
            let password_hash = hash(&sa_password, DEFAULT_COST)?;
            let new_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO users (id, tenant_id, name, email, password_hash, two_factor_enabled) VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(&new_id)
            .bind(&tenant_id)
            .bind("Super Administrateur")
            .bind(&sa_email)
            .bind(&password_hash)
            .bind(false)
            .execute(pool)
            .await?;
            println!("Created Super Admin User with email: {}", sa_email);
            new_id
        }
    };

    // 7. Link Super Admin User to Super Admin Role
    let existing_user_role = sqlx::query("SELECT 1 FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(&super_admin_user_id)
        .bind(&super_admin_role_id)
        .fetch_optional(pool)
        .await?;

    if existing_user_role.is_none() {
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
            .bind(&super_admin_user_id)
            .bind(&super_admin_role_id)
            .execute(pool)
            .await?;
        println!("Assigned Super Admin role to user: {}", sa_email);
    } else {
        println!("Super Admin role already assigned to user: {}", sa_email);
    }

    println!("Seeding completed successfully!");
    Ok(())
}
