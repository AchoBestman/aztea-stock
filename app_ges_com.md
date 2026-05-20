# AzteaStock — Spécifications Complètes de Bout en Bout

> Application SaaS multi-tenant de gestion Pharmacie & Supermarché  
> Stack : **Tauri 2 + React 18 + TypeScript** (app) · **Rust + Axum** (api) · **React Admin / Next.js** (admin)  
> Base de données cloud : **PostgreSQL via Supabase** · Local : **SQLite via sqlx**

---

## Table des matières

1. [Vision & Périmètre](#1-vision--périmètre)
2. [Organisation du projet](#2-organisation-du-projet)
3. [Modèle de données global](#3-modèle-de-données-global)
4. [Module API — Rust / Axum](#4-module-api--rust--axum)
5. [Module APP — Tauri + React](#5-module-app--tauri--react)
6. [Module ADMIN — Next.js](#6-module-admin--nextjs)
7. [Système d'abonnement & facturation](#7-système-dabonnement--facturation)
8. [Synchronisation locale ↔ cloud](#8-synchronisation-locale--cloud)
9. [Gestion des périphériques matériels](#9-gestion-des-périphériques-matériels)
10. [Sécurité & authentification](#10-sécurité--authentification)
11. [Notifications & alertes de pénurie](#11-notifications--alertes-de-pénurie)
12. [Génération PDF & impression](#12-génération-pdf--impression)
13. [Statistiques & reporting](#13-statistiques--reporting)
14. [Plan de développement par phases](#14-plan-de-développement-par-phases)
15. [Variables d'environnement](#15-variables-denvironnement)

---

## 1. Vision & Périmètre

### 1.1 Concept

AzteaStock est une application **desktop installée** chez chaque entreprise cliente (pharmacie ou supermarché), fonctionnant **100 % hors ligne** avec synchronisation automatique vers un cloud central dès qu'une connexion internet est disponible.

L'éditeur du logiciel (le "super-admin") gère les abonnements, l'activation des licences et supervise toutes les entreprises via un panneau d'administration web.

### 1.2 Trois espaces distincts

| Espace | Technologie | Utilisateur | Hébergement |
|--------|-------------|-------------|-------------|
| `app/` | Tauri 2 + React | Caissier, gérant d'entreprise | Installé localement |
| `api/` | Rust + Axum | Consommé par app et admin | VPS / Railway / Fly.io |
| `admin/` | Next.js 14 | Super-administrateur AzteaStock | Vercel / VPS |

### 1.3 Principes fondamentaux

- **Offline-first** : toutes les opérations critiques (vente, stock) fonctionnent sans internet
- **Multi-tenant** : chaque entreprise a ses données totalement isolées (tenant_id sur chaque table)
- **Abonnement mensuel** : l'app vérifie la validité de la licence au démarrage et toutes les 24h
- **Sync intelligente** : seules les données modifiées depuis la dernière sync sont envoyées (delta sync)
- **Audit trail** : toute modification de données est tracée avec horodatage et utilisateur

---

## 2. Organisation du projet

### 2.1 Structure des dossiers

```
aztea-stock/
├── app/                          # Application Tauri desktop
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── main.rs           # Point d'entrée Tauri
│   │   │   ├── commands/         # Commandes Tauri exposées au frontend
│   │   │   │   ├── mod.rs
│   │   │   │   ├── products.rs
│   │   │   │   ├── sales.rs
│   │   │   │   ├── stock.rs
│   │   │   │   ├── reports.rs
│   │   │   │   ├── sync.rs
│   │   │   │   └── settings.rs
│   │   │   ├── db/               # Couche SQLite locale
│   │   │   │   ├── mod.rs
│   │   │   │   ├── migrations/   # Fichiers SQL de migration
│   │   │   │   └── models.rs
│   │   │   ├── sync/             # Moteur de synchronisation
│   │   │   │   ├── mod.rs
│   │   │   │   ├── push.rs
│   │   │   │   └── pull.rs
│   │   │   ├── hardware/         # Drivers périphériques
│   │   │   │   ├── scanner.rs
│   │   │   │   ├── printer.rs
│   │   │   │   └── drawer.rs
│   │   │   ├── license.rs        # Vérification abonnement
│   │   │   └── notifications.rs  # Alertes locales
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   ├── src/                      # Frontend React
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── pages/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── POS.tsx           # Point de vente / caisse
│   │   │   ├── Stock.tsx
│   │   │   ├── Products.tsx
│   │   │   ├── Accounting.tsx
│   │   │   ├── Reports.tsx
│   │   │   ├── Settings.tsx
│   │   │   └── Login.tsx
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── store/                # Zustand state management
│   │   └── lib/
│   └── package.json
│
├── api/                          # API REST Rust + Axum
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── db.rs                 # Pool PostgreSQL
│   │   ├── middleware/
│   │   │   ├── auth.rs           # JWT validation + tenant extraction
│   │   │   ├── license.rs        # Vérification abonnement actif
│   │   │   └── rate_limit.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── products.rs
│   │   │   ├── sales.rs
│   │   │   ├── stock.rs
│   │   │   ├── sync.rs
│   │   │   ├── reports.rs
│   │   │   ├── subscriptions.rs
│   │   │   └── admin.rs
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── tenant.rs
│   │   │   ├── product.rs
│   │   │   ├── sale.rs
│   │   │   ├── user.rs
│   │   │   └── subscription.rs
│   │   └── errors.rs
│   ├── migrations/               # SQL migrations PostgreSQL (sqlx)
│   └── Cargo.toml
│
├── admin/                        # Panneau admin Next.js
│   ├── app/
│   │   ├── (auth)/
│   │   │   └── login/
│   │   ├── dashboard/
│   │   ├── tenants/              # Gestion des entreprises
│   │   ├── subscriptions/        # Gestion des abonnements
│   │   ├── licenses/             # Génération / révocation licences
│   │   ├── reports/              # Stats globales
│   │   └── settings/
│   ├── components/
│   ├── lib/
│   └── package.json
│
└── shared/                       # Types partagés (optionnel, TypeScript)
    └── types/
```

---

## 3. Modèle de données global

### 3.1 Tables PostgreSQL (cloud)

#### tenants — Entreprises clientes

```sql
CREATE TABLE tenants (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR(255) NOT NULL,
    business_type VARCHAR(50) NOT NULL CHECK (business_type IN ('pharmacy','supermarket','both')),
    email         VARCHAR(255) UNIQUE NOT NULL,
    phone         VARCHAR(50),
    address       TEXT,
    country       VARCHAR(100) DEFAULT 'CG',
    timezone      VARCHAR(100) DEFAULT 'Africa/Brazzaville',
    logo_url      TEXT,
    is_active     BOOLEAN DEFAULT true,
    created_at    TIMESTAMPTZ DEFAULT NOW(),
    updated_at    TIMESTAMPTZ DEFAULT NOW()
);
```

#### subscriptions — Abonnements

```sql
CREATE TABLE subscriptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plan            VARCHAR(50) NOT NULL CHECK (plan IN ('starter','pro','enterprise')),
    status          VARCHAR(50) NOT NULL CHECK (status IN ('trial','active','suspended','cancelled')),
    price_monthly   DECIMAL(10,2) NOT NULL,
    currency        VARCHAR(10) DEFAULT 'XAF',
    started_at      TIMESTAMPTZ NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    trial_ends_at   TIMESTAMPTZ,
    cancelled_at    TIMESTAMPTZ,
    payment_method  VARCHAR(100),
    notes           TEXT,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);
```

#### licenses — Clés de licence par installation

```sql
CREATE TABLE licenses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id),
    license_key     VARCHAR(64) UNIQUE NOT NULL,
    device_name     VARCHAR(255),
    device_fingerprint VARCHAR(255),
    is_active       BOOLEAN DEFAULT true,
    last_verified_at TIMESTAMPTZ,
    activated_at    TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);
```

#### users — Utilisateurs par tenant

```sql
CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    email       VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role        VARCHAR(50) NOT NULL CHECK (role IN ('owner','manager','cashier','viewer')),
    pin_hash    VARCHAR(255),           -- PIN 4 chiffres pour accès rapide caisse
    is_active   BOOLEAN DEFAULT true,
    last_login  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (tenant_id, email)
);
```

#### categories — Catégories de produits

```sql
CREATE TABLE categories (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    color       VARCHAR(7),             -- hex color pour l'UI
    icon        VARCHAR(100),
    parent_id   UUID REFERENCES categories(id),
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ             -- soft delete
);
```

#### products — Produits

```sql
CREATE TABLE products (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    category_id     UUID REFERENCES categories(id),
    barcode         VARCHAR(100),
    name            VARCHAR(500) NOT NULL,
    description     TEXT,
    brand           VARCHAR(255),
    unit            VARCHAR(50) DEFAULT 'unité',
    purchase_price  DECIMAL(12,2) DEFAULT 0,
    selling_price   DECIMAL(12,2) NOT NULL,
    tax_rate        DECIMAL(5,2) DEFAULT 0,
    image_url       TEXT,
    is_active       BOOLEAN DEFAULT true,
    requires_prescription BOOLEAN DEFAULT false,  -- pharmacie
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    UNIQUE (tenant_id, barcode)
);
```

#### stock_items — Stock par produit

```sql
CREATE TABLE stock_items (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id          UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity            DECIMAL(12,3) NOT NULL DEFAULT 0,
    quantity_reserved   DECIMAL(12,3) DEFAULT 0,
    low_stock_threshold DECIMAL(12,3) DEFAULT 5,
    unit_location       VARCHAR(255),          -- emplacement rayon
    batch_number        VARCHAR(100),
    expiry_date         DATE,                  -- pharmacie
    updated_at          TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (tenant_id, product_id)
);
```

#### stock_movements — Historique des mouvements de stock

```sql
CREATE TABLE stock_movements (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES products(id),
    user_id         UUID REFERENCES users(id),
    movement_type   VARCHAR(50) NOT NULL CHECK (movement_type IN (
                        'sale','purchase','adjustment','return','loss','initial')),
    quantity_before DECIMAL(12,3) NOT NULL,
    quantity_change DECIMAL(12,3) NOT NULL,  -- négatif = sortie
    quantity_after  DECIMAL(12,3) NOT NULL,
    reference_id    UUID,                    -- id vente ou achat lié
    note            TEXT,
    occurred_at     TIMESTAMPTZ DEFAULT NOW()
);
```

#### sales — Ventes (entêtes)

```sql
CREATE TABLE sales (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id         UUID REFERENCES users(id),
    receipt_number  VARCHAR(50) NOT NULL,
    customer_name   VARCHAR(255),
    customer_phone  VARCHAR(50),
    subtotal        DECIMAL(12,2) NOT NULL,
    tax_total       DECIMAL(12,2) DEFAULT 0,
    discount_total  DECIMAL(12,2) DEFAULT 0,
    total           DECIMAL(12,2) NOT NULL,
    amount_paid     DECIMAL(12,2) NOT NULL,
    change_given    DECIMAL(12,2) DEFAULT 0,
    payment_method  VARCHAR(50) CHECK (payment_method IN ('cash','card','mobile_money','credit')),
    status          VARCHAR(50) DEFAULT 'completed' CHECK (status IN ('completed','voided','refunded')),
    notes           TEXT,
    sold_at         TIMESTAMPTZ DEFAULT NOW(),
    created_at      TIMESTAMPTZ DEFAULT NOW()
);
```

#### sale_items — Lignes de vente

```sql
CREATE TABLE sale_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sale_id         UUID NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES products(id),
    product_name    VARCHAR(500) NOT NULL,  -- snapshot nom au moment de la vente
    product_barcode VARCHAR(100),
    quantity        DECIMAL(12,3) NOT NULL,
    unit_price      DECIMAL(12,2) NOT NULL,
    tax_rate        DECIMAL(5,2) DEFAULT 0,
    discount        DECIMAL(12,2) DEFAULT 0,
    line_total      DECIMAL(12,2) NOT NULL
);
```

#### purchases — Achats / approvisionnements

```sql
CREATE TABLE purchases (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id         UUID REFERENCES users(id),
    supplier_name   VARCHAR(255),
    supplier_phone  VARCHAR(50),
    reference       VARCHAR(100),
    total           DECIMAL(12,2) NOT NULL,
    status          VARCHAR(50) DEFAULT 'received' CHECK (status IN ('pending','received','partial','cancelled')),
    notes           TEXT,
    purchased_at    TIMESTAMPTZ DEFAULT NOW(),
    created_at      TIMESTAMPTZ DEFAULT NOW()
);
```

#### purchase_items — Lignes d'achat

```sql
CREATE TABLE purchase_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    purchase_id     UUID NOT NULL REFERENCES purchases(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES products(id),
    quantity        DECIMAL(12,3) NOT NULL,
    unit_cost       DECIMAL(12,2) NOT NULL,
    expiry_date     DATE,
    batch_number    VARCHAR(100),
    line_total      DECIMAL(12,2) NOT NULL
);
```

#### alerts — Journal des alertes de pénurie

```sql
CREATE TABLE alerts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id  UUID REFERENCES products(id),
    alert_type  VARCHAR(50) NOT NULL CHECK (alert_type IN ('low_stock','out_of_stock','expiry_soon','expired')),
    message     TEXT NOT NULL,
    threshold   DECIMAL(12,3),
    current_qty DECIMAL(12,3),
    is_read     BOOLEAN DEFAULT false,
    is_resolved BOOLEAN DEFAULT false,
    triggered_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### sync_log — Journal de synchronisation

```sql
CREATE TABLE sync_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    device_id       VARCHAR(255) NOT NULL,
    sync_type       VARCHAR(20) CHECK (sync_type IN ('push','pull','full')),
    status          VARCHAR(20) CHECK (status IN ('success','partial','failed')),
    records_pushed  INT DEFAULT 0,
    records_pulled  INT DEFAULT 0,
    error_message   TEXT,
    started_at      TIMESTAMPTZ DEFAULT NOW(),
    finished_at     TIMESTAMPTZ
);
```

### 3.2 Index essentiels

```sql
-- Performance des requêtes multi-tenant
CREATE INDEX idx_products_tenant ON products(tenant_id);
CREATE INDEX idx_products_barcode ON products(tenant_id, barcode);
CREATE INDEX idx_sales_tenant_date ON sales(tenant_id, sold_at DESC);
CREATE INDEX idx_stock_movements_product ON stock_movements(tenant_id, product_id, occurred_at DESC);
CREATE INDEX idx_alerts_tenant_unread ON alerts(tenant_id, is_read) WHERE is_read = false;

-- Sync delta : retrouver les enregistrements modifiés après une date
CREATE INDEX idx_products_updated ON products(tenant_id, updated_at);
CREATE INDEX idx_sales_created ON sales(tenant_id, created_at);
CREATE INDEX idx_stock_movements_occurred ON stock_movements(tenant_id, occurred_at);
```

### 3.3 Tables SQLite locale (miroir simplifié)

Les tables locales ont la même structure que PostgreSQL avec deux colonnes supplémentaires :

```sql
-- Ajoutées à chaque table locale
sync_status  TEXT DEFAULT 'pending' CHECK (sync_status IN ('pending','synced','conflict')),
local_only   INTEGER DEFAULT 0  -- 1 = créé hors ligne, pas encore poussé
```

---

## 4. Module API — Rust / Axum

### 4.1 Dépendances Cargo.toml

```toml
[dependencies]
axum = { version = "0.7", features = ["macros", "multipart"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-tls", "uuid", "chrono", "decimal"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
bcrypt = "0.15"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1", features = ["serde"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "compression-gzip"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
anyhow = "1"
thiserror = "1"
validator = { version = "0.18", features = ["derive"] }
```

### 4.2 Configuration et démarrage (main.rs)

```rust
// api/src/main.rs
use axum::{Router, middleware};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::compression::CompressionLayer;

mod config;
mod db;
mod errors;
mod middleware as mw;
mod models;
mod routes;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: config::Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::init();

    let config = config::Config::from_env()?;
    let db = db::create_pool(&config.database_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let state = Arc::new(AppState { db, config });

    let app = Router::new()
        .nest("/api/v1/auth", routes::auth::router())
        .nest("/api/v1/products", routes::products::router())
        .nest("/api/v1/sales", routes::sales::router())
        .nest("/api/v1/stock", routes::stock::router())
        .nest("/api/v1/sync", routes::sync::router())
        .nest("/api/v1/reports", routes::reports::router())
        .nest("/api/v1/subscriptions", routes::subscriptions::router())
        .nest("/api/v1/admin", routes::admin::router())
        .layer(middleware::from_fn(mw::auth::extract_tenant))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(CompressionLayer::new())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("API AzteaStock démarrée sur :8080");
    axum::serve(listener, app).await?;
    Ok(())
}
```

### 4.3 Middleware d'authentification JWT

```rust
// api/src/middleware/auth.rs
use axum::{extract::{Request, State}, middleware::Next, response::Response, http::StatusCode};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,          // user_id
    pub tenant_id: String,    // tenant_id
    pub role: String,
    pub exp: usize,
}

pub async fn extract_tenant(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Routes publiques : login, vérification licence
    let path = req.uri().path();
    if path.starts_with("/api/v1/auth/") || path.starts_with("/api/v1/license/verify") {
        return Ok(next.run(req).await);
    }

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
```

### 4.4 Routes — Exemples complets

#### Auth (routes/auth.rs)

```
POST   /api/v1/auth/login           -- Connexion utilisateur, retourne JWT
POST   /api/v1/auth/refresh         -- Rafraîchir le token
POST   /api/v1/auth/logout          -- Invalider le token
POST   /api/v1/auth/pin-login       -- Connexion rapide par PIN (caisse)
GET    /api/v1/auth/me              -- Profil utilisateur courant
```

Payload login :
```json
{
  "email": "manager@pharmacie-abc.com",
  "password": "MotDePasse123",
  "license_key": "AZTEASTOCK-XXXX-XXXX-XXXX"
}
```

Réponse :
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "name": "Jean Moukala",
    "role": "manager",
    "tenant_id": "uuid",
    "tenant_name": "Pharmacie ABC"
  }
}
```

#### Produits (routes/products.rs)

```
GET    /api/v1/products             -- Liste paginée avec filtres
GET    /api/v1/products/:id         -- Détail produit
GET    /api/v1/products/barcode/:code -- Recherche par code-barres
POST   /api/v1/products             -- Créer produit
PUT    /api/v1/products/:id         -- Modifier produit
DELETE /api/v1/products/:id         -- Soft delete
POST   /api/v1/products/import      -- Import CSV/JSON en masse
GET    /api/v1/products/search      -- Recherche fulltext (nom, barcode)
```

Paramètres de liste : `?page=1&per_page=50&category_id=uuid&search=paracetamol&active=true`

#### Ventes (routes/sales.rs)

```
GET    /api/v1/sales                -- Liste des ventes
GET    /api/v1/sales/:id            -- Détail vente + lignes
POST   /api/v1/sales                -- Enregistrer une vente
POST   /api/v1/sales/:id/void       -- Annuler une vente
POST   /api/v1/sales/:id/refund     -- Remboursement partiel ou total
GET    /api/v1/sales/:id/receipt    -- Données du reçu (JSON pour impression)
```

Payload création vente :
```json
{
  "customer_name": "Marie Ngoma",
  "customer_phone": "+242060000000",
  "payment_method": "cash",
  "amount_paid": 5000,
  "items": [
    {
      "product_id": "uuid",
      "quantity": 2,
      "unit_price": 1500,
      "discount": 0
    }
  ]
}
```

#### Stock (routes/stock.rs)

```
GET    /api/v1/stock                -- État du stock (tous les produits)
GET    /api/v1/stock/low            -- Produits sous seuil d'alerte
GET    /api/v1/stock/expiring       -- Produits expirant dans les 30 jours
GET    /api/v1/stock/:product_id/history -- Historique mouvements
POST   /api/v1/stock/adjust         -- Ajustement manuel de stock
POST   /api/v1/purchases            -- Enregistrer un approvisionnement
GET    /api/v1/purchases            -- Liste des approvisionnements
```

#### Synchronisation (routes/sync.rs)

```
POST   /api/v1/sync/push            -- Envoyer les données locales vers cloud
POST   /api/v1/sync/pull            -- Recevoir les mises à jour cloud
GET    /api/v1/sync/status          -- Statut de la dernière sync
POST   /api/v1/sync/full            -- Sync complète (première installation)
```

Payload push :
```json
{
  "device_id": "fingerprint-unique",
  "last_sync_at": "2024-01-15T10:30:00Z",
  "data": {
    "products": [...],
    "sales": [...],
    "stock_movements": [...],
    "purchases": [...]
  }
}
```

#### Rapports (routes/reports.rs)

```
GET    /api/v1/reports/dashboard    -- KPIs du jour (CA, nb ventes, alertes)
GET    /api/v1/reports/sales        -- CA par période
GET    /api/v1/reports/top-products -- Top N produits vendus
GET    /api/v1/reports/stock-value  -- Valeur totale du stock
GET    /api/v1/reports/movements    -- Mouvements de stock par période
GET    /api/v1/reports/accounting   -- Bilan comptable simplifié
```

Paramètres : `?from=2024-01-01&to=2024-01-31&period=day|week|month`

#### Abonnements (routes/subscriptions.rs)

```
GET    /api/v1/license/verify       -- Vérifier validité licence (public)
GET    /api/v1/subscriptions/current -- Abonnement actif du tenant
POST   /api/v1/subscriptions/renew  -- Demande de renouvellement
GET    /api/v1/subscriptions/history -- Historique des paiements
```

#### Admin (routes/admin.rs) — Protégé rôle super_admin

```
GET    /api/v1/admin/tenants                    -- Liste toutes les entreprises
POST   /api/v1/admin/tenants                    -- Créer une entreprise
PUT    /api/v1/admin/tenants/:id                -- Modifier une entreprise
POST   /api/v1/admin/tenants/:id/suspend        -- Suspendre
POST   /api/v1/admin/tenants/:id/activate       -- Réactiver
DELETE /api/v1/admin/tenants/:id                -- Supprimer

GET    /api/v1/admin/subscriptions              -- Tous les abonnements
POST   /api/v1/admin/subscriptions              -- Créer abonnement
PUT    /api/v1/admin/subscriptions/:id          -- Modifier
POST   /api/v1/admin/subscriptions/:id/cancel   -- Annuler

POST   /api/v1/admin/licenses/generate          -- Générer une clé de licence
POST   /api/v1/admin/licenses/:id/revoke        -- Révoquer une licence
GET    /api/v1/admin/licenses                   -- Toutes les licences

GET    /api/v1/admin/stats                      -- Stats globales de la plateforme
GET    /api/v1/admin/sync-logs                  -- Journal de synchronisation global
```

### 4.5 Format de réponse API standard

Succès :
```json
{
  "success": true,
  "data": { ... },
  "meta": {
    "page": 1,
    "per_page": 50,
    "total": 243,
    "total_pages": 5
  }
}
```

Erreur :
```json
{
  "success": false,
  "error": {
    "code": "PRODUCT_NOT_FOUND",
    "message": "Produit introuvable",
    "details": null
  }
}
```

### 4.6 Génération de clé de licence

```rust
// api/src/models/license.rs
use sha2::{Sha256, Digest};
use base32::Alphabet;

pub fn generate_license_key(tenant_id: &str, secret: &str) -> String {
    let raw = format!("{}-{}-{}", tenant_id, chrono::Utc::now().timestamp(), secret);
    let hash = Sha256::digest(raw.as_bytes());
    let encoded = base32::encode(Alphabet::RFC4648 { padding: false }, &hash[..15]);
    // Format : AZTEASTOCK-XXXXX-XXXXX-XXXXX
    format!(
        "AZTEASTOCK-{}-{}-{}",
        &encoded[0..5],
        &encoded[5..10],
        &encoded[10..15]
    )
}

pub async fn verify_license(pool: &PgPool, key: &str) -> Result<LicenseStatus, ApiError> {
    let license = sqlx::query_as!(
        License,
        r#"
        SELECT l.*, s.status as sub_status, s.expires_at
        FROM licenses l
        JOIN subscriptions s ON l.subscription_id = s.id
        WHERE l.license_key = $1 AND l.is_active = true
        "#,
        key
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::InvalidLicense)?;

    if license.sub_status != "active" && license.sub_status != "trial" {
        return Ok(LicenseStatus::Suspended);
    }
    if license.expires_at < chrono::Utc::now() {
        return Ok(LicenseStatus::Expired);
    }
    Ok(LicenseStatus::Valid { tenant_id: license.tenant_id })
}
```

---

## 5. Module APP — Tauri + React

### 5.1 Dépendances Tauri (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-tls", "uuid", "chrono"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
rusb = "0.9"                     # USB scanner
serialport = "4"                  # RS232 pour certaines imprimantes
printpdf = "0.7"                  # Génération PDF
escpos = "0.4"                    # Protocole ESC/POS imprimante thermique
keyring = "2"                     # Stockage sécurisé de la clé de licence
machine-uid = "0.5"               # Fingerprint de la machine
```

### 5.2 Commandes Tauri principales

#### Produits

```rust
// app/src-tauri/src/commands/products.rs

#[tauri::command]
pub async fn get_products(
    state: tauri::State<'_, AppState>,
    search: Option<String>,
    category_id: Option<String>,
    page: Option<i64>,
) -> Result<PaginatedProducts, String> { ... }

#[tauri::command]
pub async fn get_product_by_barcode(
    state: tauri::State<'_, AppState>,
    barcode: String,
) -> Result<Option<Product>, String> { ... }

#[tauri::command]
pub async fn create_product(
    state: tauri::State<'_, AppState>,
    payload: CreateProductPayload,
) -> Result<Product, String> { ... }

#[tauri::command]
pub async fn update_product(
    state: tauri::State<'_, AppState>,
    id: String,
    payload: UpdateProductPayload,
) -> Result<Product, String> { ... }
```

#### Ventes / POS

```rust
#[tauri::command]
pub async fn create_sale(
    state: tauri::State<'_, AppState>,
    payload: CreateSalePayload,
) -> Result<SaleResult, String> {
    // 1. Valider le stock disponible
    // 2. Insérer la vente en SQLite local (sync_status = 'pending')
    // 3. Décrémenter le stock local
    // 4. Créer les mouvements de stock
    // 5. Déclencher l'impression du reçu si configuré
    // 6. Marquer pour sync
}

#[tauri::command]
pub async fn scan_barcode_from_camera(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // Utilise la webcam ou le scanner HID
}
```

#### Synchronisation

```rust
#[tauri::command]
pub async fn sync_now(
    state: tauri::State<'_, AppState>,
) -> Result<SyncResult, String> {
    // 1. Vérifier la connexion internet
    // 2. Push : collecter tous les enregistrements sync_status='pending'
    // 3. Envoyer à POST /api/v1/sync/push
    // 4. En cas de succès, marquer les enregistrements sync_status='synced'
    // 5. Pull : recevoir les mises à jour depuis le cloud
    // 6. Résoudre les conflits (stratégie last-write-wins par défaut)
    // 7. Logger le résultat dans sync_log
}

#[tauri::command]
pub async fn check_internet(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    // Ping léger vers l'API
}
```

### 5.3 Gestion de la licence locale

```rust
// app/src-tauri/src/license.rs

pub struct LicenseManager {
    keyring: keyring::Entry,
}

impl LicenseManager {
    pub fn new() -> Self {
        Self {
            keyring: keyring::Entry::new("AzteaStock", "license_key").unwrap(),
        }
    }

    pub fn get_stored_key(&self) -> Option<String> {
        self.keyring.get_password().ok()
    }

    pub fn store_key(&self, key: &str) -> Result<(), String> {
        self.keyring.set_password(key).map_err(|e| e.to_string())
    }

    pub async fn verify_online(&self, api_url: &str, key: &str) -> LicenseStatus {
        let client = reqwest::Client::new();
        match client
            .get(format!("{}/api/v1/license/verify", api_url))
            .header("X-License-Key", key)
            .header("X-Device-Fingerprint", &get_device_fingerprint())
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let body: LicenseVerifyResponse = resp.json().await.unwrap();
                body.status
            }
            _ => LicenseStatus::Unknown, // Mode offline : utiliser le cache
        }
    }

    // Vérification au démarrage + toutes les 24h
    pub async fn start_periodic_check(&self, api_url: String, window: tauri::Window) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
                // Vérifier et émettre un événement vers le frontend
                window.emit("license-status-changed", status).unwrap();
            }
        });
    }
}

fn get_device_fingerprint() -> String {
    machine_uid::get().unwrap_or_else(|_| "unknown".to_string())
}
```

### 5.4 Pages React principales

#### Dashboard (pages/Dashboard.tsx)

Widgets affichés :
- CA du jour / de la semaine / du mois (avec variation vs période précédente)
- Nombre de ventes du jour
- Alertes de pénurie en cours (badge rouge si > 0)
- Top 5 produits vendus du mois
- Indicateur de synchronisation (dernière sync, statut)
- Graphique des ventes des 7 derniers jours (Recharts LineChart)

#### Point de Vente / Caisse (pages/POS.tsx)

```
┌─────────────────────────────────┬───────────────────────────────┐
│  Recherche produit / Scanner    │  Panier en cours              │
│  [🔍 Recherche ou scan ...]    │  ─────────────────────────    │
│                                 │  Paracetamol 500mg x2  3000F │
│  Résultats :                    │  Doliprane 1g      x1  1500F │
│  ┌──────────────────────────┐  │  ─────────────────────────    │
│  │ Paracetamol 500mg  1500F │  │  Sous-total :        4500F   │
│  │ Stock: 45 unités         │  │  Remise :              0F    │
│  └──────────────────────────┘  │  Total :             4500F   │
│                                 │                               │
│  Saisie quantité : [___]        │  Paiement :  [Espèces ▼]    │
│                                 │  Montant reçu : [_______]    │
│  [+ Ajouter au panier]          │  Monnaie : 500F              │
│                                 │                               │
│                                 │  [🖨 Valider & Imprimer]    │
└─────────────────────────────────┴───────────────────────────────┘
```

Comportement du scanner : l'écoute du scanner HID (mode clavier) est un `useEffect` qui intercepte les frappes rapides terminées par `Enter` et les identifie comme un scan de code-barres (> 6 caractères en < 100ms).

#### Stock (pages/Stock.tsx)

- Tableau des produits avec niveau de stock coloré (vert/orange/rouge)
- Filtre par catégorie, par statut (disponible / seuil atteint / rupture)
- Modal d'ajustement manuel de stock avec motif obligatoire
- Export CSV

#### Statistiques (pages/Reports.tsx)

Graphiques avec Recharts :
- `LineChart` : évolution du CA sur 30/90/365 jours
- `BarChart` : comparaison des ventes par catégorie
- `PieChart` : répartition des modes de paiement
- `BarChart` horizontal : top 10 produits les plus vendus
- `AreaChart` : évolution du stock dans le temps

### 5.5 Gestion du mode hors ligne (store/syncStore.ts)

```typescript
import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

interface SyncState {
  isOnline: boolean
  lastSyncAt: Date | null
  pendingCount: number
  isSyncing: boolean
  sync: () => Promise<void>
}

export const useSyncStore = create<SyncState>((set, get) => ({
  isOnline: false,
  lastSyncAt: null,
  pendingCount: 0,
  isSyncing: false,

  sync: async () => {
    if (get().isSyncing) return
    set({ isSyncing: true })
    try {
      const result = await invoke<SyncResult>('sync_now')
      set({
        lastSyncAt: new Date(),
        pendingCount: 0,
        isSyncing: false,
      })
    } catch (e) {
      set({ isSyncing: false })
    }
  }
}))
```

---

## 6. Module ADMIN — Next.js

### 6.1 Pages et fonctionnalités

| Route | Description |
|-------|-------------|
| `/dashboard` | KPIs globaux : nb tenants actifs, CA global, alertes |
| `/tenants` | Liste CRUD des entreprises clientes |
| `/tenants/:id` | Détail : abonnement, licences, utilisateurs, stats |
| `/subscriptions` | Tableau de tous les abonnements avec statuts |
| `/subscriptions/new` | Création d'un abonnement pour un tenant |
| `/licenses` | Toutes les licences générées |
| `/licenses/generate` | Générer une ou plusieurs licences |
| `/sync-logs` | Journal de synchronisation global |
| `/reports` | Revenus de la plateforme, croissance |
| `/settings` | Config globale (plans, tarifs, textes) |

### 6.2 Authentification Admin

L'admin utilise un JWT distinct avec claim `role: "super_admin"`. Stocké dans un cookie `httpOnly` (Next.js middleware). Toutes les routes `/api/v1/admin/*` de l'API Rust vérifient ce claim.

### 6.3 Tableau de bord admin

Métriques affichées :
- Nombre de tenants actifs / suspendus / en essai
- Revenus mensuels récurrents (MRR) en XAF
- Nouvelles inscriptions du mois
- Taux de renouvellement
- Dernières synchronisations (avec alertes d'échec)

---

## 7. Système d'abonnement & facturation

### 7.1 Plans disponibles

| Plan | Prix/mois | Utilisateurs | Produits max | Fonctionnalités |
|------|-----------|--------------|--------------|-----------------|
| Starter | 15 000 XAF | 2 | 500 | POS, Stock, Alertes |
| Pro | 35 000 XAF | 5 | Illimité | + Statistiques avancées, Multi-caisse |
| Enterprise | 75 000 XAF | Illimité | Illimité | + API, Support prioritaire, Multi-dépôt |

### 7.2 Workflow d'activation

```
1. Super-admin crée un tenant dans /admin/tenants/new
2. Super-admin crée un abonnement pour ce tenant
3. Super-admin génère une licence (POST /api/v1/admin/licenses/generate)
4. Clé de licence transmise au client (email ou WhatsApp)
5. Client installe l'app Tauri sur son PC
6. Au premier lancement : écran d'activation avec saisie de la clé
7. L'app envoie GET /api/v1/license/verify avec la clé + fingerprint machine
8. En cas de succès : JWT stocké localement, onboarding lancé
9. Vérification périodique toutes les 24h (si offline : grâce aux 72h de grâce)
```

### 7.3 États de licence et comportements

| État | Comportement de l'app |
|------|----------------------|
| `active` | Fonctionnement normal |
| `trial` | Fonctionnement normal + bandeau "Essai - X jours restants" |
| `suspended` | Mode lecture seule uniquement (consultation stock/historique) |
| `expired` | Affichage écran de renouvellement, aucune vente possible |
| `revoked` | Application bloquée, message de contact support |
| `offline_grace` | Fonctionnement normal si dernière vérification < 72h |

### 7.4 Gestion des paiements

Dans la version initiale, le paiement est géré manuellement par le super-admin (virement, mobile money). Le super-admin confirme le paiement dans l'interface admin, ce qui renouvelle automatiquement la subscription.

Pour une version future : intégration MTN Mobile Money API (Congo) ou Orange Money API.

---

## 8. Synchronisation locale ↔ cloud

### 8.1 Stratégie delta sync

Chaque table locale a un champ `updated_at` (TIMESTAMPTZ) et `sync_status` (pending/synced/conflict).

**Push (local → cloud) :**
1. Sélectionner tous les enregistrements où `sync_status = 'pending'`
2. Regrouper par type de table en un seul payload JSON
3. Envoyer à `POST /api/v1/sync/push` avec le timestamp de la dernière sync réussie
4. L'API insère/met à jour en PostgreSQL (UPSERT avec `ON CONFLICT (id) DO UPDATE`)
5. L'API retourne les IDs traités avec succès
6. L'app marque ces IDs `sync_status = 'synced'`

**Pull (cloud → local) :**
1. Envoyer `GET /api/v1/sync/pull?since=<last_sync_at>&device_id=<id>`
2. L'API retourne toutes les modifications faites par d'autres appareils depuis `since`
3. L'app applique les changements en SQLite local
4. En cas de conflit (même ID modifié des deux côtés) : **last-write-wins** basé sur `updated_at`

### 8.2 Déclencheurs de synchronisation

- Automatique toutes les **5 minutes** si l'app est connectée
- À chaque changement de statut réseau (détection reconnexion)
- Manuel via bouton dans l'interface
- Obligatoire avant la fermeture de l'app si `pending_count > 0`

### 8.3 Gestion des conflits

```rust
// Résolution de conflit : stratégie last-write-wins
pub fn resolve_conflict<T: HasTimestamp>(local: &T, remote: &T) -> ConflictResolution {
    if local.updated_at() > remote.updated_at() {
        ConflictResolution::KeepLocal
    } else {
        ConflictResolution::UseRemote
    }
}
```

### 8.4 Première installation (full sync)

Lors de la première activation ou réinstallation :
1. `POST /api/v1/sync/full` → télécharger l'intégralité des données du tenant
2. Populate la base SQLite locale
3. Marquer tous les enregistrements `sync_status = 'synced'`
4. Stocker `last_full_sync_at` en settings locaux

---

## 9. Gestion des périphériques matériels

### 9.1 Scanner codes-barres

Les scanners USB fonctionnent en mode HID (clavier). Dans React :

```typescript
// hooks/useScanner.ts
import { useEffect, useCallback } from 'react'

export function useScanner(onScan: (barcode: string) => void) {
  useEffect(() => {
    let buffer = ''
    let lastTime = 0

    const handleKeyPress = (e: KeyboardEvent) => {
      const now = Date.now()
      // Si délai > 50ms entre deux frappes, c'est une saisie clavier normale
      if (now - lastTime > 50 && buffer.length > 0) {
        buffer = ''
      }
      lastTime = now

      if (e.key === 'Enter' && buffer.length >= 4) {
        onScan(buffer)
        buffer = ''
        e.preventDefault()
      } else if (e.key !== 'Enter') {
        buffer += e.key
      }
    }

    window.addEventListener('keypress', handleKeyPress)
    return () => window.removeEventListener('keypress', handleKeyPress)
  }, [onScan])
}
```

### 9.2 Imprimante thermique ESC/POS

```rust
// app/src-tauri/src/hardware/printer.rs
use escpos::driver::NativeUsbDriver;
use escpos::printer::Printer;
use escpos::utils::*;

pub async fn print_receipt(receipt: &Receipt, printer_config: &PrinterConfig) -> Result<(), String> {
    let driver = NativeUsbDriver::open(
        printer_config.vendor_id,
        printer_config.product_id,
    ).map_err(|e| e.to_string())?;

    let mut printer = Printer::new(driver, Protocol::default(), None);

    printer
        .init()
        .map_err(|e| e.to_string())?
        .justify(JustifyMode::CENTER)
        .bold(true)
        .size(2, 2)
        .text(&receipt.tenant_name)
        .map_err(|e| e.to_string())?
        .bold(false)
        .size(1, 1)
        .text(&receipt.tenant_address)
        .map_err(|e| e.to_string())?
        .feed(1)
        .justify(JustifyMode::LEFT)
        .text(&format!("Reçu N°: {}", receipt.receipt_number))
        .map_err(|e| e.to_string())?
        .text(&format!("Date: {}", receipt.date))
        .map_err(|e| e.to_string())?
        .feed(1);

    for item in &receipt.items {
        printer
            .text(&format!("{:<20} {:>10}", item.name, format!("{}F", item.total)))
            .map_err(|e| e.to_string())?;
    }

    printer
        .feed(1)
        .bold(true)
        .text(&format!("TOTAL: {}F", receipt.total))
        .map_err(|e| e.to_string())?
        .bold(false)
        .text(&format!("Payé: {}F  Monnaie: {}F", receipt.paid, receipt.change))
        .map_err(|e| e.to_string())?
        .feed(3)
        .cut(CutMode::PART)
        .map_err(|e| e.to_string())?
        .print_cut()
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

### 9.3 Commande Tauri d'impression

```rust
#[tauri::command]
pub async fn print_receipt(
    state: tauri::State<'_, AppState>,
    sale_id: String,
    method: PrintMethod, // Thermal | PDF | Default
) -> Result<(), String> {
    let receipt = get_receipt_data(&state.db, &sale_id).await?;
    match method {
        PrintMethod::Thermal => hardware::printer::print_receipt(&receipt, &state.printer_config).await,
        PrintMethod::PDF => reports::generate_receipt_pdf(&receipt).await,
        PrintMethod::Default => {
            if state.printer_config.is_configured {
                hardware::printer::print_receipt(&receipt, &state.printer_config).await
            } else {
                reports::generate_receipt_pdf(&receipt).await
            }
        }
    }
}
```

---

## 10. Sécurité & authentification

### 10.1 JWT — Structure des tokens

**Token utilisateur app :**
```json
{
  "sub": "user-uuid",
  "tenant_id": "tenant-uuid",
  "role": "cashier",
  "license_key": "AZTEASTOCK-XXXXX-XXXXX-XXXXX",
  "exp": 1735689600,
  "iat": 1735603200
}
```

- Access token : durée de vie **1 heure**
- Refresh token : durée de vie **30 jours**, stocké dans `keyring` (OS keychain)
- Token super-admin : durée de vie **8 heures**, pas de refresh

### 10.2 Isolation multi-tenant

Chaque requête API extrait le `tenant_id` depuis le JWT et l'applique comme filtre sur toutes les requêtes SQL. Il est impossible d'accéder aux données d'un autre tenant.

```rust
// Middleware appliqué automatiquement sur toutes les routes protégées
pub async fn inject_tenant_filter(claims: &Claims, query: &mut QueryBuilder<Postgres>) {
    query.push(" AND tenant_id = ").push_bind(claims.tenant_id);
}
```

### 10.3 Chiffrement des données locales

La base SQLite locale est chiffrée avec **SQLCipher** (clé dérivée du fingerprint machine + secret stocké dans OS keychain). Même si le fichier est copié sur un autre poste, il ne peut pas être ouvert.

### 10.4 Niveaux d'accès

| Rôle | Dashboard | POS | Stock | Achats | Rapports | Paramètres |
|------|-----------|-----|-------|--------|----------|------------|
| `cashier` | ✓ (limité) | ✓ | Lecture | ✗ | ✗ | ✗ |
| `manager` | ✓ | ✓ | ✓ | ✓ | ✓ | Partiel |
| `owner` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `viewer` | ✓ | ✗ | Lecture | Lecture | ✓ | ✗ |

---

## 11. Notifications & alertes de pénurie

### 11.1 Moteur d'alertes local (Rust)

```rust
// app/src-tauri/src/notifications.rs

pub async fn check_stock_alerts(db: &SqlitePool, window: &tauri::Window) {
    let low_stock = sqlx::query!(
        r#"
        SELECT p.name, s.quantity, s.low_stock_threshold, p.id
        FROM stock_items s
        JOIN products p ON s.product_id = p.id
        WHERE s.quantity <= s.low_stock_threshold AND p.deleted_at IS NULL
        ORDER BY (s.quantity / s.low_stock_threshold) ASC
        "#
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    for item in &low_stock {
        // Notification système OS
        tauri_plugin_notification::NotificationExt::notification(&window.app_handle())
            .builder()
            .title("⚠️ Stock bas — AzteaStock")
            .body(&format!("{} : {} unités restantes", item.name, item.quantity))
            .show()
            .ok();
    }

    // Envoyer au frontend pour affichage dans l'UI
    window.emit("stock-alerts", low_stock).unwrap();
}

// Exécuté toutes les heures
pub fn start_alert_scheduler(db: SqlitePool, window: tauri::Window) {
    tokio::spawn(async move {
        loop {
            check_stock_alerts(&db, &window).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });
}
```

### 11.2 Types d'alertes

| Type | Condition | Priorité |
|------|-----------|---------|
| `out_of_stock` | quantity = 0 | Critique |
| `low_stock` | quantity ≤ seuil | Haute |
| `expiry_soon` | expiry_date ≤ aujourd'hui + 30 jours | Haute (pharmacie) |
| `expired` | expiry_date < aujourd'hui | Critique (pharmacie) |
| `license_expiring` | expires_at ≤ aujourd'hui + 7 jours | Haute |

---

## 12. Génération PDF & impression

### 12.1 Génération d'un reçu PDF

```rust
// app/src-tauri/src/commands/reports.rs
use printpdf::*;

pub async fn generate_receipt_pdf(receipt: &Receipt) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new("Reçu", Mm(80.0), Mm(200.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

    // En-tête
    current_layer.use_text(&receipt.tenant_name, 14.0, Mm(10.0), Mm(190.0), &font_bold);
    current_layer.use_text(&receipt.tenant_address, 9.0, Mm(10.0), Mm(183.0), &font);

    // Corps du reçu...

    let path = format!("/tmp/receipt_{}.pdf", receipt.id);
    doc.save(&mut BufWriter::new(File::create(&path).unwrap())).unwrap();

    // Ouvrir le PDF avec la visionneuse par défaut du système
    open::that(&path).map_err(|e| e.to_string())?;
    Ok(())
}
```

### 12.2 Génération des rapports PDF

Même approche pour les rapports de stock, les bilans comptables et les statistiques mensuelles. Les graphiques sont rendus en SVG puis intégrés dans le PDF.

---

## 13. Statistiques & reporting

### 13.1 Requêtes SQL clés

**CA par période :**
```sql
SELECT
    DATE_TRUNC('day', sold_at) as period,
    COUNT(*) as nb_ventes,
    SUM(total) as ca,
    AVG(total) as panier_moyen
FROM sales
WHERE tenant_id = $1
  AND sold_at BETWEEN $2 AND $3
  AND status = 'completed'
GROUP BY 1
ORDER BY 1;
```

**Top produits vendus :**
```sql
SELECT
    p.name,
    p.barcode,
    SUM(si.quantity) as qty_vendue,
    SUM(si.line_total) as ca_genere,
    COUNT(DISTINCT si.sale_id) as nb_transactions
FROM sale_items si
JOIN products p ON si.product_id = p.id
WHERE si.tenant_id = $1
  AND si.sale_id IN (
      SELECT id FROM sales WHERE sold_at BETWEEN $2 AND $3 AND status = 'completed'
  )
GROUP BY p.id, p.name, p.barcode
ORDER BY qty_vendue DESC
LIMIT $4;
```

**Valeur du stock :**
```sql
SELECT
    c.name as categorie,
    COUNT(p.id) as nb_produits,
    SUM(s.quantity * p.purchase_price) as valeur_achat,
    SUM(s.quantity * p.selling_price) as valeur_vente,
    SUM(s.quantity * (p.selling_price - p.purchase_price)) as marge_potentielle
FROM stock_items s
JOIN products p ON s.product_id = p.id
LEFT JOIN categories c ON p.category_id = c.id
WHERE p.tenant_id = $1 AND p.deleted_at IS NULL
GROUP BY c.id, c.name
ORDER BY valeur_vente DESC;
```

---

## 14. Plan de développement par phases

### Phase 1 — Fondations (Semaines 1-4)

**API (Rust) :**
- [ ] Initialisation projet Axum + SQLx + migrations PostgreSQL
- [ ] Tables : tenants, users, licenses, subscriptions
- [ ] Routes auth : login, refresh, verify-license
- [ ] Middleware JWT + extraction tenant
- [ ] Route admin : CRUD tenants + génération licences
- [ ] Déploiement sur Railway/Fly.io

**APP (Tauri) :**
- [ ] Initialisation projet Tauri 2 + React + TypeScript
- [ ] Base SQLite locale + migrations
- [ ] Écran d'activation (saisie clé de licence)
- [ ] Connexion utilisateur (JWT local)
- [ ] Layout principal + navigation

**ADMIN (Next.js) :**
- [ ] Initialisation + auth super-admin
- [ ] Page tenants : liste + création + modification
- [ ] Page licences : génération + révocation

---

### Phase 2 — Catalogue & Stock (Semaines 5-8)

**API :**
- [ ] Routes produits (CRUD + recherche barcode)
- [ ] Routes catégories
- [ ] Routes stock (état + ajustements)
- [ ] Routes mouvements de stock

**APP :**
- [ ] Page Produits : liste, création, modification, import CSV
- [ ] Page Stock : tableau avec indicateurs colorés
- [ ] Ajustement manuel de stock avec motif
- [ ] Intégration scanner HID (hook useScanner)
- [ ] Scan → recherche produit → affichage infos + prix

---

### Phase 3 — Point de Vente (Semaines 9-12)

**API :**
- [ ] Routes ventes (création, annulation, remboursement)
- [ ] Calcul automatique des mouvements de stock à la vente
- [ ] Route données reçu

**APP :**
- [ ] Page POS complète (scanner → panier → paiement)
- [ ] Impression reçu thermique (ESC/POS)
- [ ] Impression reçu PDF (fallback)
- [ ] Gestion du tiroir-caisse
- [ ] Historique des ventes de la journée

---

### Phase 4 — Synchronisation (Semaines 13-15)

**API :**
- [ ] Route sync/push avec UPSERT multi-table
- [ ] Route sync/pull (delta depuis timestamp)
- [ ] Route sync/full (première installation)
- [ ] Table sync_log + journalisation

**APP :**
- [ ] Moteur de synchronisation Rust (push + pull)
- [ ] Détection connexion/déconnexion réseau
- [ ] Sync automatique toutes les 5 minutes
- [ ] Indicateur de statut de sync dans l'UI
- [ ] Résolution de conflits last-write-wins

---

### Phase 5 — Reporting & Alertes (Semaines 16-18)

**API :**
- [ ] Routes rapports (dashboard KPIs, CA, top produits)
- [ ] Requêtes statistiques optimisées

**APP :**
- [ ] Dashboard avec graphiques Recharts
- [ ] Page Statistiques complète
- [ ] Moteur d'alertes de pénurie (scheduler Rust)
- [ ] Notifications OS pour les alertes critiques
- [ ] Génération de rapports PDF (stock, comptabilité)

---

### Phase 6 — Abonnements & Comptabilité (Semaines 19-21)

**API :**
- [ ] Routes abonnements (statut, renouvellement)
- [ ] Routes achats/approvisionnements
- [ ] Routes comptabilité (bilan simplifié)

**APP :**
- [ ] Page Comptabilité (CA, dépenses, marge)
- [ ] Page Approvisionnements (création, réception)
- [ ] Écrans de renouvellement d'abonnement
- [ ] Vérification périodique de licence (toutes les 24h)

**ADMIN :**
- [ ] Page abonnements : gestion complète
- [ ] Dashboard global avec MRR et métriques
- [ ] Journal des synchronisations

---

### Phase 7 — Finalisation (Semaines 22-24)

- [ ] Tests complets (unitaires + intégration)
- [ ] Chiffrement SQLite (SQLCipher)
- [ ] Installeur Tauri (Windows NSIS, macOS dmg, Linux AppImage)
- [ ] Documentation utilisateur
- [ ] Procédure de mise à jour automatique (Tauri updater)
- [ ] Monitoring API (logs, alertes d'erreur)

---

## 15. Variables d'environnement

### API (api/.env)

```env
DATABASE_URL=postgresql://user:password@host:5432/AzteaStock
JWT_SECRET=votre_secret_jwt_tres_long_et_aleatoire
ADMIN_JWT_SECRET=secret_different_pour_admin
LICENSE_SECRET=secret_pour_generation_licences
PORT=8080
RUST_LOG=info
CORS_ORIGINS=http://localhost:3000,https://admin-stock.azteas.com
```

### APP (app/src-tauri/src/config.rs)

```rust
pub struct AppConfig {
    pub api_url: String,           // https://api-stock.azteas.com
    pub sync_interval_secs: u64,   // 300 (5 min)
    pub license_check_secs: u64,   // 86400 (24h)
    pub offline_grace_secs: u64,   // 259200 (72h)
    pub db_path: String,           // Déterminé automatiquement par Tauri
}
```

### ADMIN (admin/.env.local)

```env
NEXT_PUBLIC_API_URL=https://api-stock.azteas.com
API_URL=https://api-stock.azteas.com
ADMIN_SECRET=votre_secret_admin
NEXTAUTH_SECRET=votre_secret_nextauth
NEXTAUTH_URL=https://admin-stock.azteas.com
```

---

*Document de spécifications AzteaStock v1.0 — Référence de développement complète*
*Projet : aztea-stock/ | API : Rust/Axum | App : Tauri 2/React | Admin : Next.js 14*






je ne vois pas le menus ge gestion des categories.

dans Statistiques il faut permettre d'avoir aussi pour une periode bien definir qui sera un intervalle de date a choisir par le user et fair aussi exportation pdf. il faut aussi avoir un bon graphique de courbe sur cette page et connaitre aussi les tops 10 produits plus vendus, les tops 10 moins vendus. la moyenne de vente, l'ecart entre moins vendus a la moyenne, l'ecart entre moyenne et plus vendu. les ruptures de stocks.

dans parametres mieux vaux recuperer dynamiquement les periphériques lié au terminal avec leur etat connecté ou deconnecté. quand le user choisir un peripherique et enregistre tout impression utilise ce periphérique.

il faut l'endroit pour les scanner aussi afin de permettre au user le scanner a utiliser par defaut pour scanner les codes.

API_BASE_URL doit etre recuperer de la valeur renseigner par Addresse ApI cloud. quand le user renseigne cette valeur il faut verifier dans le backend si cela correspond a l'api url definit pour le tenant is_system. si ca correspond accepte la modification dans l'app en local du user sinon refuse aussi le user qui fait cela doit avoir les permissions qu'il faut et il ne peut pas le faire sans envoyer une requete de demande de modification de api url qui doit etre valider par un user du tenant is_system. sur l'interface de connexion il faut permettre au user de changer api url et ce changement ne necessite pas de validation car c' est la connexion le cas ou les endpoints change ceux qui utilise l'app installer doit pouvoir changer sans renter dans l'app. cela doit etre permit si et seulement le user essaie de se connecter et obtient une erreur du genre le serveur ne peut etre atteindre dans ce cas automatiquement le champs de changement de endpoint doit apparaitre et ce ompte doit avoir les permissions pour le faire.

donc API_BASE_URL doit pas avoir de valeur par defaut dans le logiciel.si une licence est activé il faut afficher la clé d'activation actuellement je vois aucune.

interface produit je prefere une liste pour l' affiche et aussi travaille sur le modal d'ajout pour que cela soit bien centré.

toute suppression necessite une confirmation et le user n'a pas la permission de faire une choix il avoir les boutons de faire la chose griser

dans caisse meiux afficher le sproduits sous forme de liste aussi et possibilité de filtrer par categorie. donne possibilté de scanner un produit. si on scann tu lance une recherche si le produit et disponible affiche le directement dans  le panier

dashboard on doit pouvoir voir les statistiques d'une des 365 jours passé