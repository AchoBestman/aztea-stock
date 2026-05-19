# AzteaStock — REST API (Rust + Axum)

Ce projet implémente l'API REST du système **AzteaStock**, une application SaaS multi-tenant pour la gestion de pharmacies et de supermarchés. Le backend est construit en **Rust** avec le framework **Axum** et documenté via **Swagger/OpenAPI (Utoipa)**.

---

## 🚀 Configuration & Lancement

### 1. Variables d'environnement
Créez un fichier `.env` dans le répertoire `api/` :

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/azteastock
JWT_SECRET=votre-super-cle-secrete-jwt-de-32-caracteres-minimum
PORT=8080
RUST_LOG=info,api=debug

# --- Options de Base de Données ---
# DB_TYPE=postgres                     # 'postgres' ou 'sqlite'
# SQLITE_DATABASE_URL=sqlite://aztea-stock-offline.db?mode=rwc
# OFFLINE=false                        # 'true' pour forcer SQLite
```

### 2. Gestion des Bases de Données (PostgreSQL & SQLite)

L'API d'AzteaStock supporte à la fois **PostgreSQL** (idéal pour la production et le mode en ligne) et **SQLite** (conçu pour l'aspect hors-ligne).

#### A. Mode Hors-Ligne (Offline)
* **Forcer le mode hors-ligne** : Si vous définissez la variable d'environnement `OFFLINE=true` ou `OFFLINE=1`, l'API pointera directement et exclusivement sur la base de données SQLite locale définie par `SQLITE_DATABASE_URL`.
* **Tolérance aux pannes (Bascule automatique)** : Si l'API est configurée pour utiliser PostgreSQL mais que ce dernier est injoignable lors de l'initialisation, le serveur ne plante pas. Il émet un avertissement dans les logs et bascule automatiquement les requêtes sur la base SQLite locale.

#### B. Choix de la Base en Développement
En développement, vous pouvez choisir le type de base de données à utiliser :
* **Rester sur SQLite** : Définissez `DB_TYPE=sqlite` dans votre fichier `.env`.
* **Utiliser PostgreSQL** : Laissez `DB_TYPE=postgres` (ou omettez la variable) et renseignez votre `DATABASE_URL`.

### 3. Lancement du serveur de développement
Positionnez-vous dans le dossier `api/` et exécutez :

```bash
cd api
cargo run
```

Le serveur écoutera par défaut sur le port `8080` (ex: `http://localhost:8080`).

---

## 🧪 Exécution des Tests

Le projet intègre des tests unitaires et d'intégration couvrant 100% de la logique implémentée (chargement de configuration, gestion de pool de connexion, middleware de sécurité JWT, formateurs d'erreurs et contrôleurs).

Les tests sont structurés selon la convention demandée :
- `api/src/tests/routes/health.tests.rs` (Tests du contrôleur de santé)
- `api/src/tests/routes/auth.tests.rs` (Tests du contrôleur d'authentification)
- `api/src/tests/middleware/auth.tests.rs` (Tests du middleware d'authentification JWT)
- `api/src/tests/config/config.tests.rs` (Tests du parseur de configuration)
- `api/src/tests/db/db.tests.rs` (Tests du gestionnaire de connexion)
- `api/src/tests/errors/errors.tests.rs` (Tests de sérialisation des erreurs API)

Pour exécuter tous les tests :

```bash
cd api
cargo test
```

---

## 📖 Tester avec Swagger UI

Une fois le serveur démarré (`cargo run`), l'interface interactive Swagger UI est accessible à l'adresse suivante :
👉 **[http://localhost:8080/swagger-ui/](http://localhost:8080/swagger-ui/)**

La spécification OpenAPI brute (JSON) est quant à elle exposée sur :
👉 **[http://localhost:8080/api-docs/openapi.json](http://localhost:8080/api-docs/openapi.json)**

### Fonctionnalités documentées dans le Swagger

Le Swagger décrit précisément chaque endpoint :
1. **Body Request** :
   - Précise le format attendu (ex: `application/json`), les types de champs et s'ils sont obligatoires ou optionnels (marqués par un astérisque rouge dans l'UI).
2. **Body Response** :
   - Indique la structure JSON renvoyée en cas de succès (code `200 OK`) ainsi que le format des réponses d'erreur standardisées (`400 Bad Request`, `401 Unauthorized`, `404 Not Found`).
3. **Path & Query Parameters** :
   - Documente les paramètres d'URL (ex: `{id}` de type UUID) et les filtres de recherche en Query params (ex: `page`, `per_page`, `search`, etc.).
4. **Routes Protégées (Cadenas 🔒)** :
   - Les routes protégées (comme `/api/v1/products`) affichent un symbole de cadenas.
   - Pour les tester depuis Swagger UI :
     1. Effectuez un appel à `/api/v1/auth/login` avec des identifiants valides pour récupérer le token de réponse (`access_token`).
     2. Cliquez sur le bouton **Authorize** en haut à droite de Swagger UI.
     3. Renseignez la valeur du token récupéré et validez.
     4. Vous pouvez désormais tester l'ensemble des routes protégées directement depuis l'interface.
