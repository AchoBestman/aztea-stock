import { Navigate, useLocation } from "react-router-dom";
import { usePermissions } from "../hooks/usePermissions";
import Forbidden from "./Forbidden";

type Props = {
  /** Chemin de route (ex. `/pos`) — utilise ROUTE_ACCESS si défini. */
  path: string;
  /** Permissions explicites (au moins une). Prioritaire sur `path` si fourni. */
  anyOf?: string[];
  children: React.ReactNode;
};

export default function RequirePermission({ path, anyOf, children }: Props) {
  const location = useLocation();
  const { canAccessRoute, hasAny, firstAllowedPath } = usePermissions();

  const allowed = anyOf?.length
    ? hasAny(...anyOf)
    : canAccessRoute(path);

  if (allowed) return <>{children}</>;

  const fallback = firstAllowedPath();
  if (path === "/" && fallback && fallback !== "/") {
    return <Navigate to={fallback} replace state={{ from: location }} />;
  }

  return <Forbidden />;
}
