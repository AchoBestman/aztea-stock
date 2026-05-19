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
```

> 💡 **Mode Standalone (Sans DB) :** Si `DATABASE_URL` n'est pas configuré ou si la base de données est injoignable, le serveur démarrera tout de même en mode dégradé (sans planter) afin de faciliter les tests de routage et de l'interface Swagger.

### 2. Lancement du serveur de développement
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
