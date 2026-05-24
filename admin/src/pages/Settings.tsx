import { useState } from "react";
import toast from "react-hot-toast";
import { getApiBaseUrl, getDefaultApiBaseUrl, setApiBaseUrl, isTauriRuntime } from "../lib/env";

export default function Settings() {
  const [apiUrl, setApiUrl] = useState(getApiBaseUrl());

  const save = () => {
    if (!apiUrl.trim()) {
      toast.error("URL invalide");
      return;
    }
    setApiBaseUrl(apiUrl.trim());
    toast.success("URL API enregistrée");
  };

  return (
    <div className="max-w-xl space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Paramètres</h2>
        <p className="text-sm text-muted-foreground mt-1">
          Configuration locale (compatible web et future app Tauri)
        </p>
      </div>

      <section className="bg-card border border-border rounded-2xl p-6 space-y-4">
        <h3 className="font-bold">API backend</h3>
        <p className="text-sm text-muted-foreground">
          Doit inclure le préfixe <code className="text-xs">/api/v1</code>. Défaut :{" "}
          {getDefaultApiBaseUrl()}
        </p>
        <input
          className="w-full px-3 py-2 rounded-lg border border-input font-mono text-sm"
          value={apiUrl}
          onChange={(e) => setApiUrl(e.target.value)}
        />
        <button
          type="button"
          onClick={save}
          className="px-4 py-2 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer"
        >
          Enregistrer
        </button>
      </section>

      <section className="bg-card border border-border rounded-2xl p-6 text-sm text-muted-foreground space-y-2">
        <h3 className="font-bold text-foreground">Environnement</h3>
        <p>Runtime : {isTauriRuntime() ? "Tauri" : "Navigateur"}</p>
        <p>
          Le client utilise <code className="text-xs">HashRouter</code> et un stockage local
          dédié (<code className="text-xs">aztea_admin_*</code>) pour pouvoir être embarqué
          dans une WebView Tauri sans modification majeure.
        </p>
      </section>
    </div>
  );
}
