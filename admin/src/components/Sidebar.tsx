import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Building2,
  CreditCard,
  KeyRound,
  RefreshCw,
  Settings,
} from "lucide-react";
import clsx from "clsx";

const links = [
  { to: "/", label: "Tableau de bord", icon: LayoutDashboard, end: true },
  { to: "/tenants", label: "Entreprises", icon: Building2 },
  { to: "/subscriptions", label: "Abonnements", icon: CreditCard },
  { to: "/licenses", label: "Licences", icon: KeyRound },
  { to: "/sync-logs", label: "Sync", icon: RefreshCw },
  { to: "/settings", label: "Paramètres", icon: Settings },
];

export default function Sidebar() {
  return (
    <aside className="w-64 shrink-0 h-full bg-sidebar border-r border-sidebar-border flex flex-col">
      <div className="px-6 py-6 border-b border-sidebar-border">
        <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          AzteaStock
        </p>
        <h1 className="text-lg font-bold text-sidebar-foreground mt-1">Administration</h1>
      </div>
      <nav className="flex-1 p-3 space-y-1 overflow-y-auto">
        {links.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) =>
              clsx(
                "flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-semibold transition-colors",
                isActive
                  ? "bg-sidebar-primary text-sidebar-primary-foreground"
                  : "text-sidebar-foreground/80 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              )
            }
          >
            <Icon className="w-4 h-4 shrink-0" />
            {label}
          </NavLink>
        ))}
      </nav>
      <p className="px-4 py-3 text-[10px] text-muted-foreground border-t border-sidebar-border">
        Client web · prêt pour Tauri
      </p>
    </aside>
  );
}
