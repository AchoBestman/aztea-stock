import { ReactNode } from "react";
import { Server } from "lucide-react";

type AuthShellProps = {
  title: string;
  subtitle: string;
  children: ReactNode;
  showApi: boolean;
  onToggleApi: () => void;
  apiUrl: string;
  onApiUrlChange: (url: string) => void;
  currentApiUrl: string;
};

export default function AuthShell({
  title,
  subtitle,
  children,
  showApi,
  onToggleApi,
  apiUrl,
  onApiUrlChange,
  currentApiUrl,
}: AuthShellProps) {
  return (
    <div className="min-h-screen flex items-center justify-center p-4 bg-background bg-noise/5">
      <div className="w-full max-w-md bg-card border border-border rounded-3xl shadow-2xl p-8">
        <div className="text-center mb-8">
          <div className="w-14 h-14 rounded-2xl bg-brand-gradient flex items-center justify-center shadow-lg transform rotate-3 mx-auto mb-3">
            <span className="text-white font-extrabold text-2xl tracking-tight">A</span>
          </div>
          <p className="text-xs font-bold uppercase tracking-widest text-muted-foreground">
            AzteaStock
          </p>
          <h1 className="text-2xl font-bold mt-1">{title}</h1>
          <p className="text-sm text-muted-foreground mt-2">{subtitle}</p>
        </div>

        {children}

        <button
          type="button"
          onClick={onToggleApi}
          className="mt-6 w-full flex items-center justify-center gap-2 text-xs text-muted-foreground hover:text-foreground cursor-pointer"
        >
          <Server className="w-3.5 h-3.5" />
          {showApi ? "Masquer l'URL API" : "Configurer l'URL API"}
        </button>
        {showApi && (
          <div className="mt-3 space-y-2">
            <input
              className="w-full px-3 py-2 text-sm rounded-lg border border-input bg-background"
              value={apiUrl}
              onChange={(e) => onApiUrlChange(e.target.value)}
              placeholder="http://localhost:8080/api/v1"
            />
            <p className="text-[10px] text-muted-foreground">
              Serveur actuel : {currentApiUrl}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
