# AzteaStock Admin

Panneau **super-admin** (tenant système) pour gérer entreprises, abonnements et licences.

- **Stack** : Vite + React 19 + TypeScript + Tailwind 4
- **API** : `http://localhost:8080/api/v1` par défaut (voir `.env.example`)
- **Routing** : `HashRouter` (hébergement statique + futur Tauri)

## Démarrage

```bash
cd admin
cp .env.example .env
pnpm install
pnpm dev
```

Ouvrir http://localhost:5173 — se connecter avec un utilisateur du **tenant système** ayant les permissions admin (`can_read_tenant`, `can_manage_subscriptions`, etc.).

## Pages

| Route | Fonction |
|-------|----------|
| `#/` | Tableau de bord (KPIs) |
| `#/tenants` | CRUD entreprises |
| `#/tenants/:id` | Détail, licences |
| `#/subscriptions` | Abonnements |
| `#/licenses` | Liste des licences |
| `#/sync-logs` | Logs de sync par tenant |
| `#/settings` | URL API |

## Intégration Tauri (plus tard)

1. Copier `admin/src` dans un projet Tauri ou pointer `frontendDist` vers `admin/dist`.
2. Conserver `HashRouter` et `src/lib/env.ts` (`isTauriRuntime()`).
3. Optionnel : ajouter `@tauri-apps/api` pour l’empreinte appareil sur les routes protégées.
