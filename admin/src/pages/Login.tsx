import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { AlertCircle, Lock, Mail, Server } from "lucide-react";
import { useAuthStore } from "../store/authStore";
import { getApiBaseUrl, getDefaultApiBaseUrl, setApiBaseUrl } from "../lib/env";
import { ApiError } from "../lib/api";

export default function Login() {
  const navigate = useNavigate();
  const { login, verifyOtp, pending2faEmail, isAuthenticated } = useAuthStore();

  useEffect(() => {
    if (isAuthenticated) navigate("/", { replace: true });
  }, [isAuthenticated, navigate]);

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [otp, setOtp] = useState("");
  const [apiUrl, setApiUrl] = useState(getApiBaseUrl() || getDefaultApiBaseUrl());
  const [showApi, setShowApi] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (apiUrl.trim()) setApiBaseUrl(apiUrl.trim());
    setLoading(true);
    try {
      const result = await login(email, password);
      if (result === "2fa") return;
      if (result === "ok") navigate("/");
      else setError("Identifiants incorrects.");
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Erreur de connexion";
      setError(msg);
      if (err instanceof ApiError && (err.status === 0 || err.status >= 500)) setShowApi(true);
      if (err instanceof TypeError) setShowApi(true);
    } finally {
      setLoading(false);
    }
  };

  const handleOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const ok = await verifyOtp(otp);
      if (ok) navigate("/");
      else {
        setError("Code invalide ou accès non autorisé.");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Code invalide.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-4 bg-background bg-noise/5">
      <div className="w-full max-w-md bg-card border border-border rounded-3xl shadow-2xl p-8">
        <div className="text-center mb-8">
          <p className="text-xs font-bold uppercase tracking-widest text-muted-foreground">
            AzteaStock
          </p>
          <h1 className="text-2xl font-bold mt-1">Panneau d'administration</h1>
          <p className="text-sm text-muted-foreground mt-2">
            Compte du tenant système uniquement
          </p>
        </div>

        {error && (
          <div className="mb-4 flex gap-2 items-start p-3 rounded-xl bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
            <span>{error}</span>
          </div>
        )}

        {pending2faEmail ? (
          <form onSubmit={handleOtp} className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Code envoyé à <strong>{pending2faEmail}</strong>
            </p>
            <input
              className="w-full px-4 py-3 rounded-xl border border-input bg-background"
              placeholder="Code à 6 chiffres"
              value={otp}
              onChange={(e) => setOtp(e.target.value)}
              autoFocus
            />
            <button
              type="submit"
              disabled={loading}
              className="w-full py-3 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-60"
            >
              {loading ? "Vérification…" : "Valider"}
            </button>
          </form>
        ) : (
          <form onSubmit={handleLogin} className="space-y-4">
            <label className="block">
              <span className="text-sm font-medium text-muted-foreground">Email</span>
              <div className="relative mt-1">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                <input
                  type="email"
                  className="w-full pl-10 pr-4 py-3 rounded-xl border border-input bg-background"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                />
              </div>
            </label>
            <label className="block">
              <span className="text-sm font-medium text-muted-foreground">Mot de passe</span>
              <div className="relative mt-1">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                <input
                  type="password"
                  className="w-full pl-10 pr-4 py-3 rounded-xl border border-input bg-background"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                />
              </div>
            </label>
            <button
              type="submit"
              disabled={loading}
              className="w-full py-3 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-60"
            >
              {loading ? "Connexion…" : "Se connecter"}
            </button>
          </form>
        )}

        <button
          type="button"
          onClick={() => setShowApi((s) => !s)}
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
              onChange={(e) => setApiUrl(e.target.value)}
              placeholder="http://localhost:8080/api/v1"
            />
            <p className="text-[10px] text-muted-foreground">
              Serveur actuel : {getApiBaseUrl()}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
