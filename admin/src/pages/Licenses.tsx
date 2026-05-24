import { useEffect, useState } from "react";
import toast from "react-hot-toast";
import { api, type License, type Tenant } from "../lib/api";
import { formatDate } from "../lib/format";

export default function Licenses() {
  const [licenses, setLicenses] = useState<License[]>([]);
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [tenantFilter, setTenantFilter] = useState("");
  const [loading, setLoading] = useState(true);

  const load = async () => {
    setLoading(true);
    try {
      const [lics, t] = await Promise.all([
        api.licenses.list({
          per_page: 100,
          tenant_id: tenantFilter || undefined,
        }),
        api.tenants.list({ per_page: 500 }),
      ]);
      setLicenses(lics.data);
      setTenants(t.data.filter((x) => !x.is_system));
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Erreur");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, [tenantFilter]);

  const tenantName = (id: string) =>
    tenants.find((t) => t.id === id)?.name || id.slice(0, 8) + "…";

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold">Licences</h2>
          <p className="text-sm text-muted-foreground">
            Clés générées pour l&apos;activation des applications desktop
          </p>
        </div>
        <select
          className="form-select max-w-[200px]"
          value={tenantFilter}
          onChange={(e) => setTenantFilter(e.target.value)}
        >
          <option value="">Toutes les entreprises</option>
          {tenants.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>
      </div>

      <div className="bg-card border border-border rounded-2xl overflow-x-auto">
        <table className="w-full text-sm min-w-[640px]">
          <thead className="bg-muted/50 text-left">
            <tr>
              <th className="px-4 py-3 font-semibold">Clé (masquée)</th>
              <th className="px-4 py-3 font-semibold">Entreprise</th>
              <th className="px-4 py-3 font-semibold">Appareil</th>
              <th className="px-4 py-3 font-semibold">Statut</th>
              <th className="px-4 py-3 font-semibold">Créée</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                  Chargement…
                </td>
              </tr>
            ) : licenses.length === 0 ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                  Aucune licence
                </td>
              </tr>
            ) : (
              licenses.map((l) => (
                <tr key={l.id} className="border-t border-border">
                  <td className="px-4 py-3 font-mono text-xs">{l.license_key_masked}</td>
                  <td className="px-4 py-3">{tenantName(l.tenant_id)}</td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {l.device_name || "—"}
                  </td>
                  <td className="px-4 py-3">
                    {l.revoked_at
                      ? "Révoquée"
                      : l.activated_at
                        ? "Activée"
                        : "En attente"}
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {formatDate(l.created_at)}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
