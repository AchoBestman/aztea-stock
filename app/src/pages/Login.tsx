import { useState, useRef, useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { AlertCircle, Lock, Mail, ShieldCheck, ArrowLeft } from "lucide-react";
import { useAuthStore } from "../store/authStore";
import { ApiError } from "../services/api";
import { getApiBaseUrl, getDefaultApiBaseUrl, setApiBaseUrl } from "../lib/env";
import AuthShell from "../components/AuthShell";

export default function Login() {
  const { login, verifyOtp } = useAuthStore();
  const navigate = useNavigate();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [apiUrl, setApiUrl] = useState(getApiBaseUrl() || getDefaultApiBaseUrl());
  const [showApi, setShowApi] = useState(false);

  // 2FA state
  const [show2fa, setShow2fa] = useState(false);
  const [otpCode, setOtpCode] = useState(["", "", "", "", "", ""]);
  const otpRefs = useRef<(HTMLInputElement | null)[]>([]);

  // Focus first OTP input when 2FA view appears
  useEffect(() => {
    if (show2fa) {
      otpRefs.current[0]?.focus();
    }
  }, [show2fa]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (apiUrl.trim()) setApiBaseUrl(apiUrl.trim());
    setLoading(true);
    try {
      const result = await login(email, password);
      if (result === "success") {
        navigate("/");
      } else if (result === "2fa") {
        setShow2fa(true);
        setOtpCode(["", "", "", "", "", ""]);
      } else {
        setError("Identifiants incorrects.");
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Erreur de connexion";
      setError(msg);
      if (err instanceof ApiError && (err.status === 0 || err.status >= 500)) setShowApi(true);
      if (err instanceof TypeError) setShowApi(true);
    } finally {
      setLoading(false);
    }
  };

  const handleOtpChange = (index: number, value: string) => {
    if (value.length > 1) value = value[value.length - 1];
    const alphanumeric = value.replace(/[^a-zA-Z0-9]/g, "").toUpperCase();
    const newCode = [...otpCode];
    newCode[index] = alphanumeric;
    setOtpCode(newCode);
    // Auto-focus next
    if (alphanumeric && index < 5) {
      otpRefs.current[index + 1]?.focus();
    }
  };

  const handleOtpKeyDown = (index: number, e: React.KeyboardEvent) => {
    if (e.key === "Backspace" && !otpCode[index] && index > 0) {
      otpRefs.current[index - 1]?.focus();
    }
  };

  const handleOtpPaste = (e: React.ClipboardEvent) => {
    e.preventDefault();
    const pasted = e.clipboardData.getData("text").replace(/[^a-zA-Z0-9]/g, "").toUpperCase().slice(0, 6);
    const newCode = [...otpCode];
    for (let i = 0; i < 6; i++) {
      newCode[i] = pasted[i] || "";
    }
    setOtpCode(newCode);
    const focusIdx = Math.min(pasted.length, 5);
    otpRefs.current[focusIdx]?.focus();
  };

  const handleVerifyOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    const code = otpCode.join("");
    if (code.length !== 6) {
      setError("Veuillez entrer le code complet à 6 caractères.");
      return;
    }
    setError("");
    setLoading(true);
    try {
      const success = await verifyOtp(email, code);
      if (success) {
        navigate("/");
      } else {
        setError("Code de vérification incorrect.");
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Erreur de vérification";
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleBack = () => {
    setShow2fa(false);
    setOtpCode(["", "", "", "", "", ""]);
    setError("");
  };

  // ── 2FA OTP View ──────────────────────────────────────────────────────────
  if (show2fa) {
    return (
      <AuthShell
        title="Vérification en 2 étapes"
        subtitle="Un code a été envoyé à votre adresse email"
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

        <div className="flex justify-center mb-6">
          <div className="w-16 h-16 rounded-2xl bg-primary/10 flex items-center justify-center">
            <ShieldCheck className="w-8 h-8 text-primary" />
          </div>
        </div>

        <p className="text-sm text-muted-foreground text-center mb-6">
          Entrez le code à 6 caractères envoyé à <strong className="text-foreground">{email}</strong>
        </p>

        <form onSubmit={handleVerifyOtp} className="space-y-6">
          <div className="flex justify-center gap-2" onPaste={handleOtpPaste}>
            {otpCode.map((digit, i) => (
              <input
                key={i}
                ref={(el) => { otpRefs.current[i] = el; }}
                type="text"
                inputMode="text"
                maxLength={1}
                value={digit}
                onChange={(e) => handleOtpChange(i, e.target.value)}
                onKeyDown={(e) => handleOtpKeyDown(i, e)}
                className="w-11 h-13 text-center text-xl font-bold rounded-xl border border-input bg-background focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all uppercase"
              />
            ))}
          </div>

          <button
            type="submit"
            disabled={loading || otpCode.join("").length !== 6}
            className="w-full py-3 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-60"
          >
            {loading ? "Vérification…" : "Vérifier le code"}
          </button>
        </form>

        <button
          type="button"
          onClick={handleBack}
          className="mt-4 w-full flex items-center justify-center gap-2 text-sm text-muted-foreground hover:text-foreground cursor-pointer"
        >
          <ArrowLeft className="w-4 h-4" />
          Retour à la connexion
        </button>
      </AuthShell>
    );
  }

  // ── Login View ────────────────────────────────────────────────────────────
  return (
    <AuthShell
      title="Connexion"
      subtitle="Gestion de stock et facturation"
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

        <div className="flex flex-col sm:flex-row sm:justify-between gap-2 text-xs">
          <Link
            to="/forgot-password"
            className="text-primary font-medium hover:underline"
          >
            Mot de passe oublié ?
          </Link>
          <Link
            to="/reset-password"
            className="text-primary font-medium hover:underline sm:text-right"
          >
            Changer mon mot de passe
          </Link>
        </div>

        <button
          type="submit"
          disabled={loading}
          className="w-full py-3 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-60"
        >
          {loading ? "Connexion…" : "Se connecter"}
        </button>
      </form>
    </AuthShell>
  );
}
