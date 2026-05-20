import { useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { AlertCircle, Clock, Moon, Sun } from 'lucide-react';


export default function Header() {
  const location = useLocation();
  const { licenseStatus, trialDaysLeft } = useAuthStore();
  const [time, setTime] = useState(new Date());
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('fr-FR', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  const formatDate = (date: Date) => {
    return date.toLocaleDateString('fr-FR', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  // Toggle Dark Mode
  const toggleDarkMode = () => {
    const root = document.documentElement;
    if (isDark) {
      root.classList.remove('dark');
      setIsDark(false);
    } else {
      root.classList.add('dark');
      setIsDark(true);
    }
  };

  const getPageTitle = () => {
    switch (location.pathname) {
      case '/':
        return 'Tableau de bord';
      case '/pos':
        return 'Caisse & Facturation';
      case '/stock':
        return 'Mouvements & Alertes de Stock';
      case '/products':
        return 'Catalogue Produits';
      case '/reports':
        return 'Analyses de Performance';
      case '/settings':
        return 'Configuration Système';
      default:
        return 'AzteaStock';
    }
  };

  return (
    <header className="h-16 border-b border-border bg-card/60 backdrop-blur-md px-8 flex items-center justify-between select-none">
      <div className="flex items-center gap-4">
        <h2 className="text-xl font-bold text-foreground tracking-tight">{getPageTitle()}</h2>
        
        {/* Trial Grace alert banner */}
        {licenseStatus === 'trial' && (
          <div className="flex items-center gap-1.5 px-3 py-1 rounded-lg bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 text-xs font-semibold animate-pulse">
            <AlertCircle className="w-3.5 h-3.5" />
            <span>Période d'essai — {trialDaysLeft} jours restants</span>
          </div>
        )}
      </div>

      <div className="flex items-center gap-6">
        {/* Date & Time Widget */}
        <div className="flex items-center gap-3 text-sm text-muted-foreground border-r border-border pr-6">
          <Clock className="w-4 h-4 text-primary" />
          <span className="font-semibold tabular-nums text-foreground">{formatTime(time)}</span>
          <span className="text-xs capitalize font-medium hidden md:inline">{formatDate(time)}</span>
        </div>

        {/* Light/Dark Toggle */}
        <button
          onClick={toggleDarkMode}
          className="p-2 rounded-xl border border-border bg-card hover:bg-accent text-foreground transition-all duration-200 cursor-pointer shadow-sm"
          title="Basculer le thème"
        >
          {isDark ? (
            <Sun className="w-4 h-4 text-amber-500" />
          ) : (
            <Moon className="w-4 h-4 text-primary" />
          )}
        </button>
      </div>
    </header>
  );
}
