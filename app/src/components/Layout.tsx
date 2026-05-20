import { Navigate, Outlet } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import Sidebar from './Sidebar';
import Header from './Header';
import { ShieldAlert } from 'lucide-react';

export default function Layout() {
  const { isAuthenticated, licenseStatus } = useAuthStore();

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      {/* Sidebar Navigation */}
      <Sidebar />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col h-full overflow-hidden">
        {/* Top Header */}
        <Header />

        {/* Expired License Gate */}
        {(licenseStatus === 'expired' || licenseStatus === 'revoked') ? (
          <div className="flex-1 flex flex-col items-center justify-center p-8 text-center bg-background/95">
            <div className="w-16 h-16 rounded-full bg-rose-500/10 flex items-center justify-center mb-4 text-rose-500 animate-bounce">
              <ShieldAlert className="w-8 h-8" />
            </div>
            <h1 className="text-2xl font-bold text-foreground mb-2">Licence Expirée ou Révoquée</h1>
            <p className="text-muted-foreground max-w-md mb-6">
              Votre abonnement a expiré ou votre licence a été révoquée par l'administrateur. Veuillez contacter le support technique ou renouveler votre licence dans le panneau de gestion.
            </p>
            <div className="flex gap-4">
              <button 
                onClick={() => window.location.reload()}
                className="px-5 py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold shadow-md cursor-pointer hover:bg-opacity-90"
              >
                Vérifier à nouveau
              </button>
              <button 
                className="px-5 py-2.5 rounded-xl border border-border bg-card text-foreground font-semibold cursor-pointer hover:bg-accent"
              >
                Contacter le support
              </button>
            </div>
          </div>
        ) : (
          /* Page Outlet */
          <main className="flex-1 overflow-y-auto bg-background/50 bg-noise/5 p-8 relative">
            {licenseStatus === 'suspended' && (
              <div className="mb-6 p-4 rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 text-sm font-semibold flex items-center gap-2">
                <ShieldAlert className="w-5 h-5" />
                <span>Mode Lecture Seule : votre abonnement est suspendu. Impossible de créer de nouvelles ventes.</span>
              </div>
            )}
            <Outlet />
          </main>
        )}
      </div>
    </div>
  );
}
