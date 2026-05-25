import { useState } from "react";
import { Link } from "react-router-dom";
import { AlertCircle, ArrowLeft, Mail } from "lucide-react";
import { api, ApiError } from "../services/api";
import { getApiBaseUrl, getDefaultApiBaseUrl, setApiBaseUrl } from "../lib/env";
import AuthShell from "../components/AuthShell";

export default function ForgotPassword() {
  const [email, setEmail] = useState("");
  const [apiUrl, setApiUrl] = useState(getApiBaseUrl() || getDefaultApiBaseUrl());
  const [showApi, setShowApi] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setSuccess("");
    if (apiUrl.trim()) setApiBaseUrl(apiUrl.trim());
    setLoading(true);
    try {
      const res = await api.auth.forgotPassword(email.trim());
      setSuccess(res.message || "Un code a été envoyé à votre adresse email.");
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Erreur lors de l'envoi";
      setError(msg);
      if (err instanceof ApiError && (err.status === 0 || err.status >= 500)) setShowApi(true);
      if (err instanceof TypeError) setShowApi(true);
    } finally {
      setLoading(false);
    }
  };

  return (
    <AuthShell
      title="Mot de passe oublié"
      subtitle="Recevez un code OTP par email"
      showApi={showApi}
      onToggleApi={() => setShowApi((s) => !s)}
      apiUrl={apiUrl}
      onApiUrlChange={setApiUrl}
      currentApiUrl={getApiBaseUrl()}
    >
      {error && (
        <div className="mb-4 flex gap-2 items-start p-3 rounded-xl bg-destructive/10 text-destructive text-sm">
          <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
          <span>{error}</span>
        </div>
      )}
      {success && (
        <div className="mb-4 p-3 rounded-xl bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 text-sm">
          {success}
        </div>
      )}
      <form onSubmit={handleSubmit} className="space-y-4">
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
        <button
          type="submit"
          disabled={loading}
          className="w-full py-3 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-60"
        >
          {loading ? "Envoi…" : "Envoyer le code"}
        </button>
      </form>
      <div className="mt-6 flex flex-col items-center gap-2 text-sm">
        <Link
          to="/reset-password"
          className="text-primary font-medium hover:underline"
        >
          J'ai déjà un code — changer mon mot de passe
        </Link>
        <Link
          to="/login"
          className="flex items-center gap-1 text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="w-3.5 h-3.5" />
          Retour à la connexion
        </Link>
      </div>
    </AuthShell>
  );
}
