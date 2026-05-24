import { useEffect } from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import { useAuthStore } from "../store/authStore";
import Sidebar from "./Sidebar";
import Header from "./Header";

const titles: Record<string, string> = {
  "/": "Tableau de bord",
  "/tenants": "Entreprises",
  "/subscriptions": "Abonnements",
  "/licenses": "Licences",
  "/sync-logs": "Journal de synchronisation",
  "/settings": "Paramètres",
};

export default function Layout() {
  const { isAuthenticated, ensureSystemAccess, systemTenant } = useAuthStore();
  const location = useLocation();

  useEffect(() => {
    if (isAuthenticated && !systemTenant) {
      void ensureSystemAccess();
    }
  }, [isAuthenticated, systemTenant, ensureSystemAccess]);

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  const basePath =
    location.pathname.startsWith("/tenants/") && location.pathname !== "/tenants"
      ? "/tenants"
      : location.pathname.replace(/\/$/, "") || "/";

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <Header title={titles[basePath] || "Détail entreprise"} />
        <main className="flex-1 overflow-y-auto p-8 bg-background/50 bg-noise/5">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
