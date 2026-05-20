import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { Mail, Lock, AlertCircle, Database, Save } from 'lucide-react';
import { setApiBaseUrl } from '../services/api';

export default function Login() {
  const { login } = useAuthStore();
  const navigate = useNavigate();
  
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [errorMsg, setErrorMsg] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  // API base URL configuration — only revealed after a server-unreachable error
  const [apiUrl, setApiUrl] = useState(localStorage.getItem('aztea_api_base_url') || '');
  const [showApiUrl, setShowApiUrl] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMsg('');

    if (!email.trim() || !password) {
      setErrorMsg("Veuillez remplir tous les champs de connexion.");
      return;
    }

    // Save proposed API URL before attempting to log in
    if (apiUrl.trim()) {
      setApiBaseUrl(apiUrl.trim());
    }
    setIsSubmitting(true);

    try {
      const success = await login(email, password);
      if (success) {
        navigate('/');
      } else {
        setErrorMsg("Échec de la connexion. Veuillez vérifier vos identifiants.");
      }
    } catch (err: any) {
      console.error("Login attempt failed:", err);
      
      // Check if it's a network error (server unreachable)
      if (err instanceof TypeError || (err.message && (err.message.includes('fetch') || err.message.includes('Network') || err.message.includes('API Error: Not Found')))) {
        setErrorMsg("Le serveur de l'API ne peut pas être atteint à cette adresse. Veuillez vérifier l'adresse saisie.");
        setShowApiUrl(true); 
      }else if(err.status === 401 || err.status === 404){
        setErrorMsg("L'adresse de l'API Cloud est incorrecte. Veuillez vérifier l'adresse saisie.");
        setShowApiUrl(true);
      }else{
        console.log(err.message)
        setErrorMsg(err.message || "Une erreur est survenue lors de la connexion.");
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleSaveApiUrlOnly = () => {
    if (!apiUrl.trim()) {
      alert("Veuillez entrer une adresse valide.");
      return;
    }
    setApiBaseUrl(apiUrl.trim());
    alert("Adresse API Cloud enregistrée localement.");
    setErrorMsg('');
  };

  return (
    <div className="min-h-screen w-screen bg-background bg-noise/5 flex items-center justify-center p-4 relative select-none">
      
      {/* Decorative Blur Backdrops */}
      <div className="absolute top-1/4 left-1/4 w-72 h-72 rounded-full bg-primary/10 blur-[100px] pointer-events-none"></div>
      <div className="absolute bottom-1/4 right-1/4 w-80 h-80 rounded-full bg-violet-500/10 blur-[120px] pointer-events-none"></div>
      
      {/* Main Login Card */}
      <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-8 relative z-10 flex flex-col items-center">
        
        {/* Brand Logo & Slogan */}
        <div className="flex flex-col items-center text-center mb-6">
          <div className="w-14 h-14 rounded-2xl bg-brand-gradient flex items-center justify-center shadow-lg transform rotate-3 mb-3">
            <span className="text-white font-extrabold text-2xl tracking-tight">A</span>
          </div>
          <h2 className="text-xl font-extrabold tracking-tight text-foreground">AzteaStock</h2>
          <p className="text-xs text-muted-foreground font-semibold mt-1">Gestion de Stock et Facturation</p>
        </div>

        {errorMsg && (
          <div className="w-full mb-4 px-4 py-2.5 bg-rose-500/10 border border-rose-500/20 text-rose-600 dark:text-rose-400 rounded-xl text-xs font-bold flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{errorMsg}</span>
          </div>
        )}

        {/* Login Form */}
        <form onSubmit={handleSubmit} className="w-full space-y-5">
          {/* API URL Config Section (displays if empty or on error) */}
          {showApiUrl && (
            <div className="p-4 bg-accent/25 border border-border rounded-2xl space-y-2">
              <label className="text-[10px] font-extrabold text-primary uppercase block flex items-center gap-1.5">
                <Database className="w-3.5 h-3.5" />
                Adresse de l'API Cloud (Endpoint)
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="ex: http://localhost:8080/api/v1"
                  value={apiUrl}
                  onChange={(e) => setApiUrl(e.target.value)}
                  className="flex-1 px-3 py-2 bg-card border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
                <button
                  type="button"
                  onClick={handleSaveApiUrlOnly}
                  className="px-3 bg-secondary hover:bg-opacity-90 rounded-xl text-foreground flex items-center justify-center transition-colors cursor-pointer"
                  title="Enregistrer l'URL uniquement"
                >
                  <Save className="w-4 h-4" />
                </button>
              </div>
              {/* <p className="text-[9px] text-muted-foreground font-semibold leading-normal">
                Champ requis pour connecter le logiciel de caisse à votre serveur Cloud ou Local.
              </p> */}
            </div>
          )}

          {/* Email input */}
          <div className="space-y-1">
            <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Email Professionnel</label>
            <div className="relative">
              <Mail className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <input
                type="email"
                required
                placeholder="ex. superadmin@aztea.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full pl-10 pr-4 py-2.5 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
              />
            </div>
          </div>

          {/* Password field */}
          <div className="space-y-1">
            <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Mot de Passe</label>
            <div className="relative">
              <Lock className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <input
                type="password"
                required
                placeholder="••••••••"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full pl-10 pr-4 py-2.5 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
              />
            </div>
          </div>

          {/* Submit Button */}
          <button
            type="submit"
            disabled={isSubmitting}
            className="w-full py-3 bg-primary text-primary-foreground font-bold text-xs rounded-xl shadow-md cursor-pointer hover:bg-opacity-95 transition-all mt-4 disabled:bg-muted"
          >
            {isSubmitting ? 'Connexion en cours...' : 'Se Connecter'}
          </button>
        </form>

        {/* Footer Tenant ID display */}
        {/* <div className="flex items-center gap-1.5 justify-center mt-6 text-[10px] text-muted-foreground font-bold uppercase tracking-wider">
          <Store className="w-3.5 h-3.5 text-primary" />
          <span>Tenant : Système Principal Aztea</span>
        </div> */}
      </div>
    </div>
  );
}
