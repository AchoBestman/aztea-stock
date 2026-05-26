/** Permissions requises pour accéder à une route (au moins une si tableau). */
export const ROUTE_ACCESS: Record<string, string[]> = {
  "/": ["can_read_sale", "can_read_stock", "can_read_product", "can_read_alert"],
  "/pos": ["can_create_sale"],
  "/stock": ["can_read_stock"],
  "/products": ["can_read_product"],
  "/categories": ["can_read_category"],
  "/sales-history": ["can_read_sale"],
  "/reports": ["can_read_sale"],
  "/settings": [],
  "/users": ["can_read_user", "can_manage_tenant_users"],
  "/roles": ["can_read_role"],
  "/sync": ["can_read_sync_log", "can_manage_sync_log"],
};

export type NavItem = {
  to: string;
  label: string;
  icon: string;
  permissions: string[];
};

export const NAV_ITEMS: Omit<NavItem, "icon">[] = [
  { to: "/", label: "Tableau de bord", permissions: ROUTE_ACCESS["/"] },
  { to: "/pos", label: "Caisse (POS)", permissions: ROUTE_ACCESS["/pos"] },
  { to: "/stock", label: "Gestion Stock", permissions: ROUTE_ACCESS["/stock"] },
  { to: "/products", label: "Produits", permissions: ROUTE_ACCESS["/products"] },
  { to: "/categories", label: "Catégories", permissions: ROUTE_ACCESS["/categories"] },
  { to: "/sales-history", label: "Historique Ventes", permissions: ROUTE_ACCESS["/sales-history"] },
  { to: "/reports", label: "Statistiques", permissions: ROUTE_ACCESS["/reports"] },
  { to: "/settings", label: "Paramètres", permissions: ROUTE_ACCESS["/settings"] },
];

export const AUTH_NAV_ITEMS: Omit<NavItem, "icon">[] = [
  { to: "/users", label: "Utilisateurs", permissions: ROUTE_ACCESS["/users"] },
  { to: "/roles", label: "Rôles", permissions: ROUTE_ACCESS["/roles"] },
];

export function isSuperAdminRole(roles: string[]): boolean {
  return roles.some((r) => r.toLowerCase() === "super admin");
}

export function canAccessRoute(
  permissions: string[],
  roles: string[],
  path: string
): boolean {
  if (isSuperAdminRole(roles)) return true;
  const required = ROUTE_ACCESS[path];
  if (!required || required.length === 0) return true;
  return required.some((p) => permissions.includes(p));
}

export function firstAllowedPath(permissions: string[], roles: string[]): string | null {
  if (isSuperAdminRole(roles)) return "/";
  const order = [
    "/",
    "/pos",
    "/stock",
    "/products",
    "/categories",
    "/sales-history",
    "/reports",
    "/settings",
    "/users",
    "/roles",
    "/sync",
  ];
  for (const path of order) {
    if (canAccessRoute(permissions, roles, path)) return path;
  }
  return null;
}

export function hasPermission(
  permissions: string[],
  roles: string[],
  perm: string
): boolean {
  if (isSuperAdminRole(roles)) return true;
  return permissions.includes(perm);
}

export function hasAnyPermission(
  permissions: string[],
  roles: string[],
  perms: string[]
): boolean {
  if (isSuperAdminRole(roles)) return true;
  if (perms.length === 0) return true;
  return perms.some((p) => permissions.includes(p));
}

/** Écriture produit (aligné API : can_manage_product). */
export function canManageProduct(permissions: string[], roles: string[]): boolean {
  return (
    hasPermission(permissions, roles, "can_manage_product") ||
    hasAnyPermission(permissions, roles, [
      "can_create_product",
      "can_update_product",
      "can_delete_product",
    ])
  );
}
