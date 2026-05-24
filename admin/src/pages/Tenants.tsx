import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { ChevronLeft, ChevronRight, Plus, Search } from "lucide-react";
import toast from "react-hot-toast";
import {
  api,
  type CreateTenantPayload,
  type Tenant,
} from "../lib/api";
import { BUSINESS_TYPES } from "../lib/format";
import { uploadTenantAvatar } from "../lib/r2/upload";
import Badge from "../components/Badge";
import Modal from "../components/Modal";
import GeoFields, { type GeoValues } from "../components/GeoFields";
import AvatarUpload from "../components/AvatarUpload";

const emptyGeo: GeoValues = { country: "", country_name: "", city: "", timezone: "" };

const emptyForm: CreateTenantPayload = {
  name: "",
  business_type: "pharmacy",
  email: "",
  phone: "",
  address: "",
  country: "",
  city: "",
  timezone: "",
};

export default function Tenants() {
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const perPage = 15;
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState("");
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [form, setForm] = useState<CreateTenantPayload>(emptyForm);
  const [geo, setGeo] = useState<GeoValues>(emptyGeo);
  const [avatarFile, setAvatarFile] = useState<File | null>(null);
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const res = await api.tenants.list({
        search: search || undefined,
        is_active: statusFilter || undefined,
        page,
        per_page: perPage,
      });
      setTenants(res.data);
      setTotal(res.total);
      setTotalPages(res.total_pages);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Erreur de chargement");
    } finally {
      setLoading(false);
    }
  }, [search, statusFilter, page]);

  useEffect(() => {
    const t = setTimeout(() => void load(), 300);
    return () => clearTimeout(t);
  }, [load]);

  useEffect(() => {
    setPage(1);
  }, [search, statusFilter]);

  const resetModal = () => {
    setForm(emptyForm);
    setGeo(emptyGeo);
    setAvatarFile(null);
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!geo.country || !geo.city || !geo.timezone) {
      toast.error("Pays, ville et fuseau horaire sont obligatoires.");
      return;
    }
    setSaving(true);
    try {
      let logo_url: string | undefined;
      if (avatarFile) {
        const up = await uploadTenantAvatar(form.name, avatarFile);
        logo_url = up.url;
      }
      await api.tenants.create({
        ...form,
        country: geo.country_name || geo.country,
        country_code: geo.country,
        city: geo.city,
        timezone: geo.timezone,
        logo_url,
      });
      toast.success("Entreprise créée");
      setModalOpen(false);
      resetModal();
      await load();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Échec de création");
    } finally {
      setSaving(false);
    }
  };

  const toggleActive = async (t: Tenant) => {
    if (t.is_system) {
      toast.error("Le tenant système ne peut pas être suspendu.");
      return;
    }
    try {
      await api.tenants.update(t.id, { is_active: t.is_active === false });
      toast.success(t.is_active === false ? "Entreprise réactivée" : "Entreprise suspendue");
      await load();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Erreur");
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold">Entreprises</h2>
          <p className="text-sm text-muted-foreground">{total} enregistrement(s)</p>
        </div>
        <button
          type="button"
          onClick={() => {
            resetModal();
            setModalOpen(true);
          }}
          className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          Nouvelle entreprise
        </button>
      </div>

      <div className="flex flex-wrap gap-3">
        <div className="relative flex-1 min-w-[200px] max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            className="w-full pl-10 pr-4 py-2.5 rounded-xl border border-input bg-card"
            placeholder="Rechercher…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <select
          className="form-select max-w-[180px]"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
        >
          <option value="">Tous les statuts</option>
          <option value="true">Actifs</option>
          <option value="false">Suspendus</option>
        </select>
      </div>

      <div className="bg-card border border-border rounded-2xl overflow-x-auto">
        <table className="w-full text-sm min-w-[800px]">
          <thead className="bg-muted/50 text-left">
            <tr>
              <th className="px-4 py-3 font-semibold">Entreprise</th>
              <th className="px-4 py-3 font-semibold">Localisation</th>
              <th className="px-4 py-3 font-semibold">Email</th>
              <th className="px-4 py-3 font-semibold">Statut</th>
              <th className="px-4 py-3 font-semibold text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                  Chargement…
                </td>
              </tr>
            ) : tenants.length === 0 ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                  Aucun résultat
                </td>
              </tr>
            ) : (
              tenants.map((t) => (
                <tr key={t.id} className="border-t border-border hover:bg-accent/30">
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-3">
                      {t.logo_url ? (
                        <img
                          src={t.logo_url}
                          alt=""
                          className="w-9 h-9 rounded-lg object-cover border border-border"
                        />
                      ) : (
                        <div className="w-9 h-9 rounded-lg bg-muted flex items-center justify-center text-xs font-bold">
                          {t.name.charAt(0)}
                        </div>
                      )}
                      <div>
                        <Link
                          to={t.is_system ? "#" : `/tenants/${t.id}`}
                          className={`font-semibold ${t.is_system ? "text-muted-foreground" : "hover:text-primary"}`}
                        >
                          {t.name}
                        </Link>
                        {t.is_system && (
                          <Badge label="Système" tone="default" />
                        )}
                      </div>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-muted-foreground text-xs">
                    {[t.city, t.country].filter(Boolean).join(", ") || "—"}
                    {t.timezone && (
                      <span className="block text-[10px]">{t.timezone}</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">{t.email}</td>
                  <td className="px-4 py-3">
                    <Badge
                      label={t.is_active === false ? "Suspendu" : "Actif"}
                      tone={t.is_active === false ? "suspended" : "active"}
                    />
                  </td>
                  <td className="px-4 py-3 text-right">
                    {!t.is_system && (
                      <button
                        type="button"
                        onClick={() => toggleActive(t)}
                        className="text-xs font-semibold text-primary cursor-pointer"
                      >
                        {t.is_active === false ? "Réactiver" : "Suspendre"}
                      </button>
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="flex items-center justify-center gap-4">
          <button
            type="button"
            disabled={page <= 1}
            onClick={() => setPage((p) => p - 1)}
            className="p-2 rounded-lg border border-border disabled:opacity-40 cursor-pointer"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <span className="text-sm text-muted-foreground">
            Page {page} / {totalPages}
          </span>
          <button
            type="button"
            disabled={page >= totalPages}
            onClick={() => setPage((p) => p + 1)}
            className="p-2 rounded-lg border border-border disabled:opacity-40 cursor-pointer"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      )}

      {modalOpen && (
        <Modal title="Nouvelle entreprise" onClose={() => setModalOpen(false)} wide>
          <form onSubmit={handleCreate} className="space-y-4">
            <AvatarUpload
              previewUrl={null}
              onFileSelect={setAvatarFile}
              disabled={saving}
            />
            <input
              required
              placeholder="Nom commercial"
              className="form-input"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
            <select
              className="form-select"
              value={form.business_type}
              onChange={(e) => setForm({ ...form, business_type: e.target.value })}
            >
              {BUSINESS_TYPES.map((b) => (
                <option key={b.value} value={b.value}>
                  {b.label}
                </option>
              ))}
            </select>
            <input
              required
              type="email"
              placeholder="Email"
              className="form-input"
              value={form.email}
              onChange={(e) => setForm({ ...form, email: e.target.value })}
            />
            <input
              placeholder="Téléphone"
              className="form-input"
              value={form.phone || ""}
              onChange={(e) => setForm({ ...form, phone: e.target.value })}
            />
            <textarea
              placeholder="Adresse (rue, quartier…)"
              className="form-input min-h-[60px]"
              value={form.address || ""}
              onChange={(e) => setForm({ ...form, address: e.target.value })}
            />
            <GeoFields value={geo} onChange={setGeo} disabled={saving} />
            <button
              type="submit"
              disabled={saving}
              className="w-full py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-60"
            >
              {saving ? "Création…" : "Créer"}
            </button>
          </form>
        </Modal>
      )}
    </div>
  );
}
