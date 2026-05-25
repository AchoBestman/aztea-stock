import { NavLink, Link } from 'react-router-dom';
import { 
  LayoutDashboard, 
  ShoppingCart, 
  Package, 
  BarChart3, 
  Settings, 
  LogOut, 
  RefreshCw, 
  Wifi, 
  WifiOff,
  User,
  ShieldCheck,
  Tag,
  Receipt
} from 'lucide-react';
import { useAuthStore } from '../store/authStore';
import { useSyncStore } from '../store/syncStore';
import { getBusinessTypeLabel } from '../lib/format';
import clsx from 'clsx';

export default function Sidebar() {
  const { user, logout, licenseStatus } = useAuthStore();
  const { isOnline, isSyncing } = useSyncStore();

  const menuItems = [
    { to: '/', icon: LayoutDashboard, label: 'Tableau de bord' },
    { to: '/pos', icon: ShoppingCart, label: 'Caisse (POS)' },
    { to: '/stock', icon: Package, label: 'Gestion Stock' },
    { to: '/products', icon: Package, label: 'Produits' },
    { to: '/categories', icon: Tag, label: 'Catégories' },
    { to: '/sales-history', icon: Receipt, label: 'Historique Ventes' },
    { to: '/reports', icon: BarChart3, label: 'Statistiques' },
    { to: '/settings', icon: Settings, label: 'Paramètres' },
  ];

  return (
    <aside className="w-64 bg-sidebar border-r border-sidebar-border flex flex-col h-screen select-none">
      {/* Brand Header — tenant */}
      <div className="p-6 border-b border-sidebar-border flex items-center justify-between">
        <div className="flex items-center gap-3 min-w-0">
          <div className="w-10 h-10 rounded-xl bg-brand-gradient flex items-center justify-center shadow-lg shrink-0 overflow-hidden">
            {user?.tenantLogoUrl ? (
              <img
                src={user.tenantLogoUrl}
                alt=""
                className="w-full h-full object-cover"
              />
            ) : (
              <span className="text-white font-extrabold text-xl tracking-tight">
                {(user?.tenantName ?? 'A').charAt(0).toUpperCase()}
              </span>
            )}
          </div>
          <div className="min-w-0">
            <h1 className="font-bold text-lg text-foreground tracking-tight truncate">
              {user?.tenantName ?? 'Établissement'}
            </h1>
            <span className="text-xs text-muted-foreground font-medium">
              {getBusinessTypeLabel(user?.tenantBusinessType ?? 'pharmacy')}
            </span>
          </div>
        </div>
      </div>

      {/* Connection & License Badges */}
      <div className="px-6 py-4 border-b border-sidebar-border bg-muted/30 space-y-2.5">
        {/* Sync / Online Status */}
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-2">
            <div className="relative flex h-2 w-2">
              <span className={clsx(
                "animate-ping absolute inline-flex h-full w-full rounded-full opacity-75",
                isOnline ? "bg-emerald-400" : "bg-rose-400"
              )}></span>
              <span className={clsx(
                "relative inline-flex rounded-full h-2 w-2",
                isOnline ? "bg-emerald-500" : "bg-rose-500"
              )}></span>
            </div>
            <span className="font-medium text-muted-foreground">
              {isOnline ? 'Mode En Ligne' : 'Mode Hors Ligne'}
            </span>
          </div>
          {isOnline ? (
            <Wifi className="w-3.5 h-3.5 text-emerald-500" />
          ) : (
            <WifiOff className="w-3.5 h-3.5 text-rose-500" />
          )}
        </div>

        {/* License status */}
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-1.5 text-muted-foreground">
            <ShieldCheck className="w-3.5 h-3.5 text-primary" />
            <span className="font-medium">Licence</span>
          </div>
          <span className={clsx(
            "px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider",
            licenseStatus === 'active' && "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
            licenseStatus === 'trial' && "bg-amber-500/10 text-amber-600 dark:text-amber-400",
            (licenseStatus === 'expired' || licenseStatus === 'revoked') && "bg-rose-500/10 text-rose-600 dark:text-rose-400"
          )}>
            {licenseStatus}
          </span>
        </div>
      </div>

      {/* Menu Navigation */}
      <nav className="flex-1 px-4 py-6 space-y-6 overflow-y-auto">
        <div className="space-y-1.5">
          {menuItems.map((item) => {
            const Icon = item.icon;
            return (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) => clsx(
                  "flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200 group card-hover",
                  isActive 
                    ? "bg-primary dark:bg-blue-600 text-primary-foreground shadow-md glow-primary" 
                    : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
                )}
              >
                <Icon className="w-5 h-5 transition-transform duration-300 group-hover:scale-110" />
                <span>{item.label}</span>
              </NavLink>
            );
          })}
        </div>

        {user && (user.role === 'admin' || user.role === 'Super Admin') && (
          <div className="space-y-1.5">
            <h3 className="px-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Auth</h3>
            <NavLink
              to="/users"
              className={({ isActive }) => clsx(
                "flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200 group card-hover",
                isActive 
                  ? "bg-primary dark:bg-blue-600 text-primary-foreground shadow-md glow-primary" 
                  : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              )}
            >
              <User className="w-5 h-5 transition-transform duration-300 group-hover:scale-110" />
              <span>Utilisateurs</span>
            </NavLink>
            <NavLink
              to="/roles"
              className={({ isActive }) => clsx(
                "flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200 group card-hover",
                isActive 
                  ? "bg-primary dark:bg-blue-600 text-primary-foreground shadow-md glow-primary" 
                  : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              )}
            >
              <ShieldCheck className="w-5 h-5 transition-transform duration-300 group-hover:scale-110" />
              <span>Rôles</span>
            </NavLink>
          </div>
        )}
      </nav>

      {/* Sync Button & User Section */}
      <div className="p-4 border-t border-sidebar-border bg-muted/20 space-y-3">
        {/* Sync CTA */}
        <Link
          to="/sync"
          className={clsx(
            "w-full flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl text-xs font-semibold border border-border shadow-sm cursor-pointer transition-all duration-300",
            isSyncing 
              ? "bg-muted text-muted-foreground cursor-not-allowed" 
              : "bg-card text-foreground hover:bg-accent"
          )}
        >
          <RefreshCw className={clsx("w-3.5 h-3.5 text-primary dark:text-blue-400", isSyncing && "animate-spin")} />
          <span>{isSyncing ? 'Synchronisation...' : 'Synchroniser'}</span>

        </Link>

        {/* User Card */}
        {user && (
          <div className="flex items-center gap-3 p-2.5 rounded-xl bg-card border border-border">
            <div className="w-9 h-9 rounded-lg bg-accent flex items-center justify-center text-primary border border-border">
              <User className="w-4 h-4 dark:text-blue-400" />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs font-semibold text-foreground truncate">{user.name}</p>
              <p className="text-[10px] text-muted-foreground font-medium capitalize truncate">
                {user.role}
              </p>
            </div>
            <button 
              onClick={logout}
              className="text-muted-foreground hover:text-destructive p-1 rounded-lg transition-colors"
              title="Se déconnecter"
            >
              <LogOut className="w-4 h-4" />
            </button>
          </div>
        )}
      </div>
    </aside>
  );
}
