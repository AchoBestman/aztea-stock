import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import toast from "react-hot-toast";
import {
  api,
  type CreateSubscriptionPayload,
  type Subscription,
  type Tenant,
} from "../lib/api";
import {
  formatCurrency,
  formatDate,
  PLAN_PRESETS,
  SUBSCRIPTION_STATUSES,
} from "../lib/format";
import Badge from "../components/Badge";
import Modal from "../components/Modal";

function defaultExpires(): string {
  const d = new Date();
  d.setMonth(d.getMonth() + 1);
  return d.toISOString().slice(0, 16);
}

export default function Subscriptions() {
  const [items, setItems] = useState<Subscription[]>([]);
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [form, setForm] = useState<CreateSubscriptionPayload>({
    tenant_id: "",
    plan: "starter",
    status: "trial",
    price_monthly: PLAN_PRESETS.starter.price,
    currency: "XAF",
    max_devices: 1,
    expires_at: defaultExpires(),
  });

  const load = async () => {
    setLoading(true);
    try {
      const [subs, t] = await Promise.all([
        api.subscriptions.list({ per_page: 200 }),
        api.tenants.list({ per_page: 500 }),
      ]);
      setItems(subs.data);
      setTenants(t.data.filter((x) => !x.is_system));
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Erreur");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const tenantName = (id: string) =>
    tenants.find((t) => t.id === id)?.name || id.slice(0, 8) + "…";

  const onPlanChange = (plan: string) => {
    const preset = PLAN_PRESETS[plan];
    setForm((f) => ({
      ...f,
      plan,
      price_monthly: preset?.price ?? f.price_monthly,
      currency: preset?.currency ?? "XAF",
      max_devices: preset?.max_devices ?? f.max_devices,
    }));
  };

  const openCreate = () => {
    setEditingId(null);
    setForm({
      tenant_id: "",
      plan: "starter",
      status: "trial",
      price_monthly: PLAN_PRESETS.starter.price,
      currency: "XAF",
      max_devices: 1,
      expires_at: defaultExpires(),
    });
    setModalOpen(true);
  };

  const openEdit = (s: Subscription) => {
    setEditingId(s.id);
    setForm({
      tenant_id: s.tenant_id,
      plan: s.plan,
      status: s.status,
      price_monthly: typeof s.price_monthly === 'string' ? parseFloat(s.price_monthly) : s.price_monthly,
      currency: s.currency,
      max_devices: s.max_devices,
      expires_at: new Date(s.expires_at).toISOString().slice(0, 16),
      notes: s.notes || "",
    });
    setModalOpen(true);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const expires = new Date(form.expires_at).toISOString();
      if (editingId) {
        await api.subscriptions.update(editingId, {
          ...form,
          expires_at: expires,
        });
        toast.success("Abonnement mis à jour");
      } else {
        await api.subscriptions.create({
          ...form,
          expires_at: expires,
        });
        toast.success("Abonnement créé");
      }
      setModalOpen(false);
      await load();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Erreur");
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold">Abonnements</h2>
          <p className="text-sm text-muted-foreground">{items.length} enregistrement(s)</p>
        </div>
        <button
          type="button"
          onClick={openCreate}
          className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          Nouvel abonnement
        </button>
      </div>

      <div className="bg-card border border-border rounded-2xl overflow-x-auto">
        <table className="w-full text-sm min-w-[720px]">
          <thead className="bg-muted/50 text-left">
            <tr>
              <th className="px-4 py-3 font-semibold">Entreprise</th>
              <th className="px-4 py-3 font-semibold">Plan</th>
              <th className="px-4 py-3 font-semibold">Statut</th>
              <th className="px-4 py-3 font-semibold">Prix</th>
              <th className="px-4 py-3 font-semibold">Appareils Max</th>
              <th className="px-4 py-3 font-semibold">Expiration</th>
              <th className="px-4 py-3 font-semibold text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-muted-foreground">
                  Chargement…
                </td>
              </tr>
            ) : (
              items.map((s) => (
                <tr key={s.id} className="border-t border-border group">
                  <td className="px-4 py-3 font-medium">{tenantName(s.tenant_id)}</td>
                  <td className="px-4 py-3 capitalize">{s.plan}</td>
                  <td className="px-4 py-3">
                    <Badge label={s.status} tone={tone(s.status)} />
                  </td>
                  <td className="px-4 py-3">{formatCurrency(s.price_monthly, s.currency)}</td>
                  <td className="px-4 py-3 font-mono">{s.max_devices}</td>
                  <td className="px-4 py-3 text-muted-foreground">{formatDate(s.expires_at)}</td>
                  <td className="px-4 py-3 text-right">
                    <button
                      onClick={() => openEdit(s)}
                      className="px-3 py-1 rounded-lg bg-secondary text-secondary-foreground text-xs font-semibold hover:bg-secondary/80 transition-colors"
                    >
                      Modifier
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {modalOpen && (
        <Modal 
          title={editingId ? "Modifier l'abonnement" : "Nouvel abonnement"} 
          onClose={() => setModalOpen(false)} 
          wide
        >
          <form onSubmit={handleSubmit} className="space-y-4">
            <select
              required
              disabled={!!editingId}
              className="form-select disabled:opacity-50"
              value={form.tenant_id}
              onChange={(e) => setForm({ ...form, tenant_id: e.target.value })}
            >
              <option value="">Entreprise</option>
              {tenants.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
            </select>
            <select
              className="form-select"
              value={form.plan}
              onChange={(e) => onPlanChange(e.target.value)}
            >
              {Object.entries(PLAN_PRESETS).map(([k, v]) => (
                <option key={k} value={k}>
                  {v.label} — {v.price.toLocaleString()} {v.currency}
                </option>
              ))}
            </select>
            <select
              className="form-select"
              value={form.status}
              onChange={(e) => setForm({ ...form, status: e.target.value })}
            >
              {SUBSCRIPTION_STATUSES.map((s) => (
                <option key={s.value} value={s.value}>
                  {s.label}
                </option>
              ))}
            </select>
            <input
              type="number"
              className="form-input"
              value={form.price_monthly}
              onChange={(e) =>
                setForm({ ...form, price_monthly: parseFloat(e.target.value) })
              }
            />
            <div className="space-y-1">
              <label className="text-xs font-bold text-muted-foreground uppercase">Nombre d&apos;appareils maximum</label>
              <input
                type="number"
                min={1}
                required
                className="form-input"
                value={form.max_devices}
                onChange={(e) =>
                  setForm({ ...form, max_devices: parseInt(e.target.value) || 1 })
                }
              />
            </div>
            <label className="block text-sm text-muted-foreground">
              Date d&apos;expiration
              <input
                type="datetime-local"
                className="form-input mt-1"
                value={form.expires_at}
                onChange={(e) => setForm({ ...form, expires_at: e.target.value })}
              />
            </label>
            <textarea
              placeholder="Notes (optionnel)"
              className="form-input min-h-[60px]"
              value={form.notes || ""}
              onChange={(e) => setForm({ ...form, notes: e.target.value })}
            />
            <button
              type="submit"
              className="w-full py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer"
            >
              {editingId ? "Enregistrer" : "Créer"}
            </button>
          </form>
        </Modal>
      )}
    </div>
  );
}

function tone(status: string) {
  if (status === "active" || status === "production") return "active" as const;
  if (status === "trial") return "trial" as const;
  if (status === "suspended") return "suspended" as const;
  return "cancelled" as const;
}
