import { useEffect, useState, type ReactNode } from "react";
import { Link, useParams } from "react-router-dom";
import { ArrowLeft, KeyRound, Mail, Pencil, Power, ShieldPlus, Trash2, UserPlus } from "lucide-react";
import toast from "react-hot-toast";
import {
  api,
  type AdminUser,
  type GroupedPermission,
  type License,
  type Role,
  type Subscription,
  type Tenant,
  type UpdateTenantPayload,
} from "../lib/api";
import { formatCurrency, formatDate, BUSINESS_TYPES } from "../lib/format";
import { uploadTenantAvatar } from "../lib/r2/upload";
import Badge from "../components/Badge";
import Modal from "../components/Modal";
import GeoFields, { type GeoValues } from "../components/GeoFields";
import AvatarUpload from "../components/AvatarUpload";
import LogoUrlField from "../components/LogoUrlField";
import { useR2UploadAvailable } from "../hooks/useR2Upload";
import { useAuthStore } from "../store/authStore";
import { Switch } from "@/components/Switch";
import ConfirmModal from "../components/ConfirmModal";

export default function TenantDetail() {
  const r2UploadAvailable = useR2UploadAvailable();
  const { id } = useParams<{ id: string }>();
  const [tenant, setTenant] = useState<Tenant | null>(null);
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [licenses, setLicenses] = useState<License[]>([]);
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [roleModal, setRoleModal] = useState(false);
  const [newRole, setNewRole] = useState({ name: "", description: "" });
  const [groupedPerms, setGroupedPerms] = useState<GroupedPermission[]>([]);
  const [selectedPerms, setSelectedPerms] = useState<Set<string>>(new Set());
  const [savingRole, setSavingRole] = useState(false);
  const [editRoleId, setEditRoleId] = useState<string | null>(null);
  const [uploadProgress, setUploadProgress] = useState<number | null>(null);
  const [editOpen, setEditOpen] = useState(false);
  const [form, setForm] = useState<UpdateTenantPayload>({});
  const [geo, setGeo] = useState<GeoValues>({ country: "", country_name: "", city: "", timezone: "" });
  const [avatarFile, setAvatarFile] = useState<File | null>(null);
  const [licenseModal, setLicenseModal] = useState(false);
  const [userModal, setUserModal] = useState(false);
  const [selectedSubId, setSelectedSubId] = useState("");
  const [generatedKey, setGeneratedKey] = useState<string | null>(null);
  const [newUser, setNewUser] = useState({ name: "", email: "", role_id: "" });
  const [revealedKeys, setRevealedKeys] = useState<Record<string, string>>({});
  const [viewRolePerms, setViewRolePerms] = useState<Role | null>(null);
  const [confirmData, setConfirmData] = useState<{
    title: string;
    message: string;
    onConfirm: () => void;
  } | null>(null);
  const { user: currentUser } = useAuthStore();

  const hasPermission = (p: string) => currentUser?.permissions.includes(p) || currentUser?.roles.includes("Super Admin");

  const load = async () => {
    if (!id) return;
    try {
      const [t, subs, lics, us, rs] = await Promise.all([
        api.tenants.get(id),
        api.subscriptions.list({ tenant_id: id, per_page: 50 }),
        api.licenses.list({ tenant_id: id, per_page: 50 }),
        api.users.list(id),
        api.roles.list(id),
      ]);
      setTenant(t);
      setForm({
        name: t.name,
        business_type: t.business_type,
        email: t.email,
        phone: t.phone ?? undefined,
        address: t.address ?? undefined,
        sender_email: t.sender_email ?? undefined,
        sender_user: t.sender_user_encrypted ? "********" : "",
        sender_password: "",
      });
      setGeo({
        country: t.country_code || t.country || "",
        country_name: t.country || "",
        city: t.city || "",
        timezone: t.timezone || "",
      });
      setSubscriptions(subs.data);
      setLicenses(lics.data);
      setUsers(us);
      setRoles(rs);
    } catch (e) {
      toast.error(getErrMsg(e));
    }
  };

  useEffect(() => {
    void load();
  }, [id]);

  const getErrMsg = (err: unknown): string => {
    if (err instanceof Error) return err.message;
    if (typeof err === "object" && err !== null) {
      const e = err as Record<string, unknown>;
      if (typeof e.message === "string") return e.message;
      if (typeof e.error === "string") return e.error;
    }
    return "Une erreur est survenue";
  };

  const save = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id || !tenant) return;
    if (!geo.country || !geo.city || !geo.timezone) {
      toast.error("Pays, ville et fuseau horaire sont obligatoires.");
      return;
    }
    let logo_url: string | undefined;
    if (r2UploadAvailable && avatarFile) {
      setUploadProgress(0);
      try {
        const up = await uploadTenantAvatar(
          form.name || tenant.name,
          avatarFile,
          (p) => setUploadProgress(p)
        );
        logo_url = up.url;
      } catch (uploadErr) {
        toast.error("Logo non sauvegardé : " + getErrMsg(uploadErr));
        logo_url = tenant.logo_url ?? undefined;
      } finally {
        setUploadProgress(null);
      }
    } else {
      logo_url =
        (form.logo_url as string | undefined) ?? tenant.logo_url ?? undefined;
    }
    const updatePayload: UpdateTenantPayload = {
      ...form,
      country: geo.country_name || geo.country,
      country_code: geo.country,
      city: geo.city,
      timezone: geo.timezone,
      logo_url,
    };

    // Don't send "********" back to server (unmodified sensitive field)
    if (updatePayload.sender_user === "********") {
      delete updatePayload.sender_user;
    }
    if (!updatePayload.sender_password) {
      delete updatePayload.sender_password;
    }

    try {
      const updated = await api.tenants.update(id, updatePayload);
      setTenant(updated);
      setEditOpen(false);
      setAvatarFile(null);
      toast.success("Entreprise mise à jour");
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const toggleUser = async (user: AdminUser) => {
    try {
      await api.users.updateStatus(user.id, !(user.is_active ?? true));
      const us = await api.users.list(id!);
      setUsers(us);
      toast.success(
        (user.is_active ?? true) ? `${user.name} désactivé` : `${user.name} activé`
      );
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const toggleUser2FA = async (user: AdminUser) => {
    try {
      await api.users.setTwoFactor(user.id, !user.two_factor_enabled);
      const us = await api.users.list(id!);
      setUsers(us);
      toast.success(
        user.two_factor_enabled ? `2FA désactivé pour ${user.name}` : `2FA activé pour ${user.name}`
      );
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const toggleTenant2FA = async () => {
    if (!tenant || !id) return;
    try {
      const updated = await api.tenants.setTwoFactor(id, !tenant.two_factor_enabled);
      setTenant(updated);
      toast.success(
        tenant.two_factor_enabled ? "2FA désactivé pour le tenant" : "2FA activé pour le tenant"
      );
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const sendResetLink = async (user: AdminUser) => {
    try {
      await api.users.sendReset(user.email);
      toast.success(`Lien envoyé à ${user.email}`);
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const deleteSubscription = async (subId: string) => {
    setConfirmData({
      title: "Supprimer l'abonnement",
      message: "Êtes-vous sûr de vouloir supprimer cet abonnement ? Cette action est irréversible.",
      onConfirm: async () => {
        try {
          await api.subscriptions.delete(subId);
          const subs = await api.subscriptions.list({ tenant_id: id!, per_page: 50 });
          setSubscriptions(subs.data);
          toast.success("Abonnement supprimé");
        } catch (err) {
          toast.error(getErrMsg(err));
        }
      },
    });
  };

  const changeSubStatus = async (subId: string, status: string) => {
    try {
      await api.subscriptions.updateStatus(subId, status);
      const subs = await api.subscriptions.list({ tenant_id: id!, per_page: 50 });
      setSubscriptions(subs.data);
      toast.success("Statut mis à jour");
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const revealLicense = async (licId: string) => {
    try {
      const res = await api.licenses.reveal(licId);
      setRevealedKeys((prev) => ({ ...prev, [licId]: res.license_key_plain }));
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const sendLicenseKey = async (licId: string) => {
    try {
      await api.licenses.sendKey(licId);
      toast.success("Clé envoyée par email au tenant");
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const generateLicense = async () => {
    if (!id || !selectedSubId) return;
    try {
      const lic = await api.licenses.generate(id, selectedSubId);
      setGeneratedKey(lic.license_key_plain);
      const refreshed = await api.licenses.list({ tenant_id: id, per_page: 50 });
      setLicenses(refreshed.data);
      toast.success("Licence générée");
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const createAdminUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id) return;
    try {
      await api.users.create({
        name: newUser.name,
        email: newUser.email,
        role_id: newUser.role_id,
        tenant_id: id,
      });
      toast.success("Utilisateur créé — invitation envoyée par email");
      setUserModal(false);
      setNewUser({ name: "", email: "", role_id: "" });
      const us = await api.users.list(id);
      setUsers(us);
    } catch (err) {
      toast.error(getErrMsg(err));
    }
  };

  const openRoleModal = async (roleToEdit?: Role) => {
    if (groupedPerms.length === 0) {
      try {
        const g = await api.permissions.listGrouped();
        setGroupedPerms(g);
      } catch { /* non-bloquant */ }
    }
    if (roleToEdit) {
      setEditRoleId(roleToEdit.id);
      setNewRole({ name: roleToEdit.name, description: roleToEdit.description ?? "" });
      try {
        const perms = await api.roles.getPermissions(roleToEdit.id);
        setSelectedPerms(new Set(perms.map((p) => p.id)));
      } catch {
        setSelectedPerms(new Set());
      }
    } else {
      setEditRoleId(null);
      setNewRole({ name: "", description: "" });
      setSelectedPerms(new Set());
    }
    setRoleModal(true);
  };

  const submitRole = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id) return;
    setSavingRole(true);
    try {
      let role: Role;
      if (editRoleId) {
        role = await api.roles.update(editRoleId, {
          name: newRole.name,
          description: newRole.description || undefined,
        });
        await api.roles.setPermissions(role.id, [...selectedPerms]);
        toast.success(`Rôle « ${role.name} » modifié`);
      } else {
        role = await api.roles.create({
          name: newRole.name,
          description: newRole.description || undefined,
          tenant_id: id,
        });
        if (selectedPerms.size > 0) {
          await api.roles.setPermissions(role.id, [...selectedPerms]);
        }
        toast.success(`Rôle « ${role.name} » créé`);
      }
      setRoleModal(false);
      const rs = await api.roles.list(id);
      setRoles(rs);
    } catch (err) {
      toast.error(getErrMsg(err));
    } finally {
      setSavingRole(false);
    }
  };

  const deleteRole = async (roleId: string, roleName: string) => {
    setConfirmData({
      title: "Supprimer le rôle",
      message: `Êtes-vous sûr de vouloir supprimer le rôle "${roleName}" ? Il ne doit être attribué à aucun utilisateur.`,
      onConfirm: async () => {
        try {
          await api.roles.delete(roleId);
          toast.success(`Rôle "${roleName}" supprimé`);
          const rs = await api.roles.list(id!);
          setRoles(rs);
        } catch (err) {
          toast.error(getErrMsg(err));
        }
      },
    });
  };

  const togglePerm = (permId: string) =>
    setSelectedPerms((prev) => {
      const next = new Set(prev);
      if (next.has(permId)) next.delete(permId); else next.add(permId);
      return next;
    });

  const SYSTEM_ONLY_GROUPS = ["tenants", "cross-tenant", "licenses", "subscriptions"];
  const visiblePerms = tenant
    ? tenant.is_system
      ? groupedPerms
      : groupedPerms.filter((g) => !SYSTEM_ONLY_GROUPS.includes(g.group))
    : groupedPerms;

  if (!tenant) {
    return <p className="text-muted-foreground">Chargement…</p>;
  }

  return (
    <div className="space-y-6">
      <Link
        to="/tenants"
        className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="w-4 h-4" />
        Retour aux entreprises
      </Link>

      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="flex gap-4 items-start">
          {tenant.logo_url ? (
            <img
              src={tenant.logo_url}
              alt=""
              className="w-16 h-16 rounded-2xl object-cover border border-border"
            />
          ) : (
            <div className="w-16 h-16 rounded-2xl bg-muted flex items-center justify-center text-xl font-bold">
              {tenant.name.charAt(0)}
            </div>
          )}
          <div>
            <h2 className="text-2xl font-bold">{tenant.name}</h2>
            <p className="text-muted-foreground text-sm mt-1">{tenant.email}</p>
            <div className="mt-2 flex gap-2">
              <Badge
                label={tenant.is_active === false ? "Suspendu" : "Actif"}
                tone={tenant.is_active === false ? "suspended" : "active"}
              />
              {tenant.is_system && <Badge label="Système" tone="default" />}
              {tenant.two_factor_enabled && <Badge label="2FA" tone="default" />}
            </div>
          </div>
        </div>
        <div className="flex gap-2 flex-wrap items-center">
          {hasPermission("can_manage_two_factor_for_tenant") && (
            <div className="flex items-center gap-2 mr-4 border border-border p-2 rounded-xl">
              <span className="text-xs font-semibold">2FA Tenant</span>
              <Switch checked={tenant.two_factor_enabled} onChange={() => void toggleTenant2FA()} />
            </div>
          )}
          <button
            type="button"
            onClick={() => setEditOpen(true)}
            className="px-4 py-2 rounded-xl border border-border font-semibold text-sm cursor-pointer hover:bg-accent"
          >
            Modifier
          </button>
          <button
            type="button"
            onClick={() => setUserModal(true)}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-xl border border-border font-semibold text-sm cursor-pointer hover:bg-accent"
          >
            <UserPlus className="w-4 h-4" />
            Admin utilisateur
          </button>
          <button
            type="button"
            onClick={() => void openRoleModal()}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-xl border border-border font-semibold text-sm cursor-pointer hover:bg-accent"
          >
            <ShieldPlus className="w-4 h-4" />
            Nouveau rôle
          </button>
          {!tenant.is_system && (
            <button
              type="button"
              onClick={() => setLicenseModal(true)}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-primary text-primary-foreground font-semibold text-sm cursor-pointer"
            >
              <KeyRound className="w-4 h-4" />
              Générer licence
            </button>
          )}
        </div>
      </div>

      <div className="grid md:grid-cols-2 gap-4 text-sm">
        <Info label="Type" value={tenant.business_type} />
        <Info label="Email" value={tenant.email} />
        <Info label="Téléphone" value={tenant.phone || "—"} />
        <Info label="Ville" value={tenant.city || "—"} />
        <Info
          label="Pays"
          value={
            tenant.country && tenant.country_code
              ? `${tenant.country} (${tenant.country_code})`
              : tenant.country || tenant.country_code || "—"
          }
        />
        <Info label="Fuseau" value={tenant.timezone || "—"} />
        <Info label="Adresse" value={tenant.address || "—"} />
        <Info label="Créé le" value={formatDate(tenant.created_at)} />
      </div>

      <Section title="Rôles">
        {roles.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            Aucun rôle.{" "}
            <button
              type="button"
              onClick={() => void openRoleModal()}
              className="text-primary underline cursor-pointer"
            >
              Créer le premier rôle
            </button>
          </p>
        ) : (
          <ul className="space-y-2 text-sm">
            {roles.map((r) => (
              <li
                key={r.id}
                className="flex flex-col gap-2 p-3 rounded-xl border border-border"
              >
                <div className="flex items-center justify-between gap-2">
                  <span>
                    <span className="font-semibold">{r.name}</span>
                    {r.description && (
                      <span className="text-xs text-muted-foreground ml-2">{r.description}</span>
                    )}
                  </span>
                  <div className="flex gap-1">
                    <button
                      type="button"
                      title="Modifier ce rôle"
                      onClick={() => void openRoleModal(r)}
                      className="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground cursor-pointer"
                    >
                      <Pencil className="w-5 h-5" />
                    </button>
                    {hasPermission("can_delete_role") && r.name !== "Super Admin" && (
                      <button
                        type="button"
                        title="Supprimer ce rôle"
                        onClick={() => void deleteRole(r.id, r.name)}
                        className="p-1.5 rounded-lg hover:bg-accent text-destructive cursor-pointer"
                      >
                        <Trash2 className="w-5 h-5" />
                      </button>
                    )}
                  </div>
                </div>
                {r.permissions && r.permissions.length > 0 && (
                  <div className="flex flex-wrap items-center gap-1.5 mt-1">
                    {r.permissions.slice(0, 5).map((p) => (
                      <span key={p.id} className="text-[10px] px-2 py-0.5 rounded-md bg-muted text-muted-foreground font-medium">
                        {p.name}
                      </span>
                    ))}
                    {r.permissions.length > 3 && (
                      <button
                        type="button"
                        onClick={() => setViewRolePerms(r)}
                        className="text-[10px] px-2 py-0.5 rounded-md bg-primary/10 text-primary font-bold hover:bg-primary/20 cursor-pointer"
                      >
                        +{r.permissions.length - 3}
                      </button>
                    )}
                  </div>
                )}
              </li>
            ))}
          </ul>
        )}
      </Section>

      <Section title="Utilisateurs">
        {users.length === 0 ? (
          <p className="text-sm text-muted-foreground">Aucun utilisateur.</p>
        ) : (
          <ul className="space-y-2 text-sm">
            {users.map((u) => (
              <li
                key={u.id}
                className="flex flex-wrap items-center justify-between gap-2 p-3 rounded-xl border border-border"
              >
                <span className={u.is_active === false ? "opacity-50" : ""}>
                  <span className="font-semibold">{u.name}</span>
                  <span className="text-muted-foreground"> — {u.email}</span>
                  {u.roles.length > 0 && (
                    <span className="ml-2 text-xs text-muted-foreground">[{u.roles.join(", ")}]</span>
                  )}
                </span>
                <span className="flex items-center gap-1">
                  {hasPermission("can_manage_two_factor_for_user") && (
                    <div className="flex items-center gap-2 mr-2 border border-border p-1.5 rounded-lg">
                      <span className="text-[10px] font-bold uppercase text-muted-foreground">2FA</span>
                      <Switch checked={u.two_factor_enabled} onChange={() => void toggleUser2FA(u)} />
                    </div>
                  )}
                  <button
                    type="button"
                    title="Envoyer lien de réinitialisation"
                    onClick={() => void sendResetLink(u)}
                    className="p-2 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground cursor-pointer"
                  >
                    <Mail className="w-5 h-5" />
                  </button>
                  <button
                    type="button"
                    title={u.is_active === false ? "Activer" : "Désactiver"}
                    onClick={() => void toggleUser(u)}
                    className={`p-2 rounded-lg hover:bg-accent cursor-pointer ${
                      u.is_active === false
                        ? "text-green-600"
                        : "text-destructive hover:text-destructive"
                    }`}
                  >
                    <Power className="w-5 h-5" />
                  </button>
                </span>
              </li>
            ))}
          </ul>
        )}
      </Section>

      {!tenant.is_system && (
        <Section title="Abonnements">
          {subscriptions.length === 0 ? (
            <p className="text-sm text-muted-foreground">Aucun abonnement.</p>
          ) : (
            <ul className="space-y-2">
              {subscriptions.map((s) => (
                <li
                  key={s.id}
                  className="flex flex-wrap items-center justify-between gap-2 p-3 rounded-xl border border-border"
                >
                  <div className="flex items-center gap-3 flex-wrap">
                    <span className="font-semibold capitalize">{s.plan}</span>
                    <Badge label={s.status} tone={statusTone(s.status)} />
                    <span className="text-muted-foreground">
                      {formatCurrency(s.price_monthly, s.currency)}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      Expire {formatDate(s.expires_at)}
                    </span>
                  </div>
                  <div className="flex items-center gap-1">
                    <select
                      className="text-xs rounded-lg border border-border bg-background px-2 py-1 cursor-pointer"
                      value={s.status}
                      onChange={(e) => void changeSubStatus(s.id, e.target.value)}
                      title="Modifier le statut"
                    >
                      <option value="trial">Trial</option>
                      <option value="active">Actif</option>
                      <option value="suspended">Suspendu</option>
                      <option value="cancelled">Annulé</option>
                    </select>
                    <button
                      type="button"
                      title="Supprimer"
                      onClick={() => void deleteSubscription(s.id)}
                      className="p-1.5 rounded-lg hover:bg-accent text-destructive cursor-pointer"
                    >
                      ×
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Section>
      )}

      {!tenant.is_system && (
        <Section title="Licences">
          {licenses.length === 0 ? (
            <p className="text-sm text-muted-foreground">Aucune licence.</p>
          ) : (
            <ul className="space-y-2 font-mono text-xs">
              {licenses.map((l) => (
                <li
                  key={l.id}
                  className="p-3 rounded-xl border border-border flex flex-wrap items-center justify-between gap-2"
                >
                  <div className="flex flex-col gap-1">
                    <span>{revealedKeys[l.id] ?? l.license_key_masked}</span>
                    <span className="text-muted-foreground">
                      {l.activated_at ? "Activée" : "En attente"}
                    </span>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      type="button"
                      title={revealedKeys[l.id] ? "Clé révélée" : "Voir la clé"}
                      onClick={() => void revealLicense(l.id)}
                      className="px-2 py-1 rounded-lg border border-border hover:bg-accent text-[10px] uppercase tracking-wide cursor-pointer"
                    >
                      {revealedKeys[l.id] ? "Clé affichée" : "Voir clé"}
                    </button>
                    <button
                      type="button"
                      title="Envoyer au tenant par email"
                      onClick={() => void sendLicenseKey(l.id)}
                      className="px-2 py-1 rounded-lg border border-border hover:bg-accent text-[10px] uppercase tracking-wide inline-flex items-center gap-1 cursor-pointer"
                    >
                      <Mail className="w-3 h-3" />
                      Envoyer
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Section>
      )}

      {editOpen && (
        <Modal title="Modifier l'entreprise" onClose={() => setEditOpen(false)} wide>
          <form onSubmit={save} className="space-y-3">
            {r2UploadAvailable === null ? (
              <p className="text-xs text-muted-foreground">Vérification du stockage…</p>
            ) : r2UploadAvailable ? (
              <AvatarUpload
                previewUrl={tenant.logo_url}
                onFileSelect={setAvatarFile}
                progress={uploadProgress}
              />
            ) : (
              <LogoUrlField
                value={(form.logo_url as string | undefined) ?? tenant.logo_url ?? ""}
                onChange={(logo_url) => setForm({ ...form, logo_url })}
              />
            )}
            <input
              className="form-input"
              value={form.name || ""}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
            <select
              className="form-select"
              value={form.business_type || ""}
              onChange={(e) => setForm({ ...form, business_type: e.target.value })}
            >
              {BUSINESS_TYPES.map((b) => (
                <option key={b.value} value={b.value}>
                  {b.label}
                </option>
              ))}
            </select>
            <input
              type="email"
              className="form-input"
              value={form.email || ""}
              onChange={(e) => setForm({ ...form, email: e.target.value })}
            />
            <input
              className="form-input"
              value={(form.phone as string) || ""}
              onChange={(e) => setForm({ ...form, phone: e.target.value })}
            />
            <textarea
              className="form-input min-h-[60px]"
              value={(form.address as string) || ""}
              onChange={(e) => setForm({ ...form, address: e.target.value })}
            />
            {tenant.is_system && hasPermission("can_update_tenant_credentials") && currentUser?.tenant_id === tenant.id && (
              <div className="space-y-3 p-4 rounded-xl border border-border bg-muted/30">
                <p className="text-xs font-bold uppercase tracking-wide text-muted-foreground">Configuration SMTP (Système uniquement)</p>
                <div className="space-y-2">
                  <label className="block">
                    <span className="text-xs font-semibold ml-1">Email de l'expéditeur</span>
                    <input
                      className="form-input mt-1"
                      placeholder="ex: noreply@aztea.com"
                      value={form.sender_email || ""}
                      onChange={(e) => setForm({ ...form, sender_email: e.target.value })}
                    />
                  </label>
                  <label className="block">
                    <span className="text-xs font-semibold ml-1">Nom d'utilisateur SMTP</span>
                    <input
                      className="form-input mt-1"
                      value={form.sender_user || ""}
                      onChange={(e) => setForm({ ...form, sender_user: e.target.value })}
                    />
                  </label>
                  <label className="block">
                    <span className="text-xs font-semibold ml-1">Mot de passe SMTP</span>
                    <input
                      type="password"
                      className="form-input mt-1"
                      placeholder="Laisser vide pour ne pas modifier"
                      value={form.sender_password || ""}
                      onChange={(e) => setForm({ ...form, sender_password: e.target.value })}
                    />
                  </label>
                </div>
              </div>
            )}
            <GeoFields value={geo} onChange={setGeo} />
            <button
              type="submit"
              className="w-full py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer"
            >
              Enregistrer
            </button>
          </form>
        </Modal>
      )}

      {userModal && (
        <Modal title="Créer un administrateur" onClose={() => setUserModal(false)}>
          <form onSubmit={createAdminUser} className="space-y-4">
            <input
              required
              placeholder="Nom"
              className="form-input"
              value={newUser.name}
              onChange={(e) => setNewUser({ ...newUser, name: e.target.value })}
            />
            <input
              required
              type="email"
              placeholder="Email"
              className="form-input"
              value={newUser.email}
              onChange={(e) => setNewUser({ ...newUser, email: e.target.value })}
            />
            <select
              required
              className="form-select"
              value={newUser.role_id}
              onChange={(e) => setNewUser({ ...newUser, role_id: e.target.value })}
            >
              <option value="">— Choisir un rôle —</option>
              {roles.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.name}
                </option>
              ))}
            </select>
            {roles.length === 0 && (
              <p className="text-xs text-amber-600">
                Aucun rôle pour ce tenant.{" "}
                <button
                  type="button"
                  className="underline"
                  onClick={() => { setUserModal(false); void openRoleModal(); }}
                >
                  Créer un rôle d&apos;abord
                </button>
              </p>
            )}
            <button
              type="submit"
              disabled={!newUser.role_id}
              className="w-full py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-50"
            >
              Inviter
            </button>
          </form>
        </Modal>
      )}

      {roleModal && (
        <Modal
          title={editRoleId ? "Modifier le rôle" : "Nouveau rôle"}
          onClose={() => setRoleModal(false)}
          wide
        >
          <form onSubmit={(e) => void submitRole(e)} className="space-y-4">
            <input
              required
              placeholder="Nom du rôle (ex : Gérant, Caissier…)"
              className="form-input"
              value={newRole.name}
              onChange={(e) => setNewRole({ ...newRole, name: e.target.value })}
            />
            <input
              placeholder="Description (optionnel)"
              className="form-input"
              value={newRole.description}
              onChange={(e) => setNewRole({ ...newRole, description: e.target.value })}
            />

            {visiblePerms.length > 0 && (
              <div className="space-y-3 max-h-80 overflow-y-auto pr-1">
                <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                  Permissions ({selectedPerms.size} sélectionnées)
                </p>
                {visiblePerms.map((g) => (
                  <div key={g.group} className="rounded-xl border border-border p-3 space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-bold uppercase tracking-wide text-muted-foreground">
                        {g.group}
                      </span>
                      <button
                        type="button"
                        className="text-xs text-primary underline cursor-pointer"
                        onClick={() => {
                          const allSelected = g.permissions.every((p) => selectedPerms.has(p.id));
                          setSelectedPerms((prev) => {
                            const s = new Set(prev);
                            g.permissions.forEach((p) =>
                              allSelected ? s.delete(p.id) : s.add(p.id)
                            );
                            return s;
                          });
                        }}
                      >
                        {g.permissions.every((p) => selectedPerms.has(p.id)) ? "Tout désélect." : "Tout sélect."}
                      </button>
                    </div>
                    {g.permissions.map((p) => (
                      <label key={p.id} className="flex items-start gap-2 text-sm cursor-pointer">
                        <input
                          type="checkbox"
                          className="mt-0.5 accent-primary"
                          checked={selectedPerms.has(p.id)}
                          onChange={() => togglePerm(p.id)}
                        />
                        <span>
                          <span className="font-medium">{p.name}</span>
                          {p.description && (
                            <span className="text-xs text-muted-foreground block">{p.description}</span>
                          )}
                        </span>
                      </label>
                    ))}
                  </div>
                ))}
              </div>
            )}

            <button
              type="submit"
              disabled={savingRole || !newRole.name.trim()}
              className="w-full py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-60"
            >
              {savingRole ? "Enregistrement…" : editRoleId ? "Enregistrer" : "Créer le rôle"}
            </button>
          </form>
        </Modal>
      )}

      {licenseModal && (
        <Modal
          title="Générer une licence"
          onClose={() => {
            setLicenseModal(false);
            setGeneratedKey(null);
          }}
        >
          {generatedKey ? (
            <div className="space-y-4">
              <p className="text-sm text-muted-foreground">
                Copiez cette clé — elle ne sera plus affichée en clair.
              </p>
              <code className="block p-4 rounded-xl bg-muted font-mono text-sm break-all">
                {generatedKey}
              </code>
              <button
                type="button"
                onClick={() => navigator.clipboard.writeText(generatedKey)}
                className="w-full py-2 rounded-xl border font-semibold cursor-pointer"
              >
                Copier
              </button>
            </div>
          ) : (
            <div className="space-y-4">
              <select
                className="form-select"
                value={selectedSubId}
                onChange={(e) => setSelectedSubId(e.target.value)}
              >
                <option value="">Choisir un abonnement</option>
                {subscriptions
                  .filter((s) => s.status === "active" || s.status === "trial")
                  .map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.plan} — {s.status}
                    </option>
                  ))}
              </select>
              <button
                type="button"
                disabled={!selectedSubId}
                onClick={generateLicense}
                className="w-full py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold cursor-pointer disabled:opacity-50"
              >
                Générer
              </button>
            </div>
          )}
        </Modal>
      )}

      {viewRolePerms && (
        <Modal
          title={`Permissions — ${viewRolePerms.name}`}
          onClose={() => setViewRolePerms(null)}
          wide
        >
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              {viewRolePerms.description || "Aucune description."}
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-96 overflow-y-auto pr-1">
              {viewRolePerms.permissions?.map((p) => (
                <div key={p.id} className="p-3 rounded-lg border border-border bg-muted/20">
                  <p className="text-xs font-bold text-foreground">{p.name}</p>
                  {p.description && <p className="text-[10px] text-muted-foreground mt-0.5">{p.description}</p>}
                </div>
              ))}
            </div>
            <button
              type="button"
              onClick={() => setViewRolePerms(null)}
              className="w-full py-2 rounded-xl border border-border font-semibold cursor-pointer"
            >
              Fermer
            </button>
          </div>
        </Modal>
      )}

      <ConfirmModal
        isOpen={!!confirmData}
        title={confirmData?.title || ""}
        message={confirmData?.message || ""}
        onConfirm={() => confirmData?.onConfirm()}
        onCancel={() => setConfirmData(null)}
      />
    </div>
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="p-4 rounded-xl border border-border bg-card">
      <p className="text-xs text-muted-foreground font-medium">{label}</p>
      <p className="mt-1 font-medium">{value}</p>
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="bg-card border border-border rounded-2xl p-5">
      <h3 className="font-bold mb-4">{title}</h3>
      {children}
    </section>
  );
}

function statusTone(status: string) {
  if (status === "active") return "active" as const;
  if (status === "trial") return "trial" as const;
  if (status === "suspended") return "suspended" as const;
  return "cancelled" as const;
}
