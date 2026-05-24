import { useEffect, useState } from "react";
import toast from "react-hot-toast";
import { api, type SyncLog, type Tenant } from "../lib/api";
import { formatDate } from "../lib/format";
import Badge from "../components/Badge";

export default function SyncLogs() {
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [tenantId, setTenantId] = useState("");
  const [logs, setLogs] = useState<SyncLog[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    api.tenants
      .list({ per_page: 500 })
      .then((res) => {
        const clients = res.data.filter((x) => !x.is_system);
        setTenants(clients);
        if (clients[0]) setTenantId(clients[0].id);
      })
      .catch(() => toast.error("Impossible de charger les entreprises"));
  }, []);

  useEffect(() => {
    if (!tenantId) return;
    setLoading(true);
    api.sync
      .logs({ tenant_id: tenantId, per_page: 50 })
      .then((res) => setLogs(res.data))
      .catch((e) =>
        toast.error(e instanceof Error ? e.message : "Erreur sync logs")
      )
      .finally(() => setLoading(false));
  }, [tenantId]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Journal de synchronisation</h2>
        <p className="text-sm text-muted-foreground mt-1">
          Logs par entreprise (filtrage tenant requis par l&apos;API)
        </p>
      </div>

      <select
        className="form-select max-w-md"
        value={tenantId}
        onChange={(e) => setTenantId(e.target.value)}
      >
        {tenants.map((t) => (
          <option key={t.id} value={t.id}>
            {t.name}
          </option>
        ))}
      </select>

      <div className="bg-card border border-border rounded-2xl overflow-x-auto">
        <table className="w-full text-sm min-w-[720px]">
          <thead className="bg-muted/50 text-left">
            <tr>
              <th className="px-4 py-3 font-semibold">Appareil</th>
              <th className="px-4 py-3 font-semibold">Type</th>
              <th className="px-4 py-3 font-semibold">Statut</th>
              <th className="px-4 py-3 font-semibold">Push / Pull</th>
              <th className="px-4 py-3 font-semibold">Début</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                  Chargement…
                </td>
              </tr>
            ) : logs.length === 0 ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                  Aucun log pour cette entreprise
                </td>
              </tr>
            ) : (
              logs.map((log) => (
                <tr key={log.id} className="border-t border-border">
                  <td className="px-4 py-3 font-mono text-xs">{log.device_id}</td>
                  <td className="px-4 py-3 capitalize">{log.sync_type || "—"}</td>
                  <td className="px-4 py-3">
                    <Badge
                      label={log.status || "—"}
                      tone={
                        log.status === "success"
                          ? "success"
                          : log.status === "failed"
                            ? "failed"
                            : "partial"
                      }
                    />
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {log.records_pushed} / {log.records_pulled}
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {formatDate(log.started_at)}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
        {logs.some((l) => l.error_message) && (
          <div className="p-4 border-t border-border text-xs text-destructive space-y-2">
            {logs
              .filter((l) => l.error_message)
              .map((l) => (
                <p key={l.id}>
                  {l.device_id}: {l.error_message}
                </p>
              ))}
          </div>
        )}
      </div>
    </div>
  );
}
