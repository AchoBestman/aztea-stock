import { useAuthStore } from "../store/authStore";
import {
  canAccessRoute,
  canManageProduct,
  firstAllowedPath,
  hasAnyPermission,
  hasPermission,
  isSuperAdminRole,
} from "../lib/permissions";

export function usePermissions() {
  const permissions = useAuthStore((s) => s.permissions);
  const roles = useAuthStore((s) => s.roles);

  return {
    permissions,
    roles,
    isSuperAdmin: isSuperAdminRole(roles),
    has: (perm: string) => hasPermission(permissions, roles, perm),
    hasAny: (...perms: string[]) => hasAnyPermission(permissions, roles, perms),
    canAccessRoute: (path: string) => canAccessRoute(permissions, roles, path),
    firstAllowedPath: () => firstAllowedPath(permissions, roles),
    canManageProduct: () => canManageProduct(permissions, roles),
  };
}
