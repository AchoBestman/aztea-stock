import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Building2, CreditCard, KeyRound, TrendingUp, AlertTriangle } from "lucide-react";
import { api, type License, type Subscription, type Tenant } from "../lib/api";
import { formatCurrency, formatDate } from "../lib/format";
import StatCard from "../components/StatCard";
import Badge from "../components/Badge";

export default function Dashboard() {
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [licenses, setLicenses] = useState<License[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [t, subs, lics] = await Promise.all([
          api.tenants.list({ per_page: 500 }),
          api.subscriptions.list({ per_page: 500 }),
          api.licenses.list({ per_page: 20 }),
        ]);
        setTenants(t.data.filter((x) => !x.is_system));
        setSubscriptions(subs.data);
        setLicenses(lics.data);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const stats = useMemo(() => {
    const activeTenants = tenants.filter((t) => t.is_active !== false).length;
    const suspendedTenants = tenants.filter((t) => t.is_active === false).length;
    const activeSubs = subscriptions.filter((s) => s.status === "active");
    const trialSubs = subscriptions.filter((s) => s.status === "trial");
    const mrr = activeSubs.reduce(
      (sum, s) => sum + parseFloat(String(s.price_monthly)),
      0
    );
    const expiringSoon = subscriptions.filter((s) => {
      if (s.status !== "active" && s.status !== "trial") return false;
      const days =
        (new Date(s.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24);
      return days >= 0 && days <= 7;
    });
    return {
      activeTenants,
      suspendedTenants,
      activeSubs: activeSubs.length,
      trialSubs: trialSubs.length,
      mrr,
      expiringSoon,
      activatedLicenses: licenses.filter((l) => l.activated_at).length,
    };
  }, [tenants, subscriptions, licenses]);

  if (loading) {
    return <p className="text-muted-foreground">Chargement du tableau de bord…</p>;
  }

  return (
    <div className="space-y-8 animate-slide-up">
      <div>
        <h2 className="text-2xl font-bold">Vue plateforme</h2>
        <p className="text-muted-foreground text-sm mt-1">
          Supervision des entreprises, abonnements et licences
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
        <StatCard
          title="Entreprises actives"
          value={stats.activeTenants}
          hint={`${stats.suspendedTenants} suspendue(s)`}
          icon={Building2}
        />
        <StatCard
          title="MRR (abonnements actifs)"
          value={formatCurrency(stats.mrr)}
          hint={`${stats.activeSubs} abonnement(s) actif(s)`}
          icon={TrendingUp}
        />
        <StatCard
          title="En période d'essai"
          value={stats.trialSubs}
          icon={CreditCard}
        />
        <StatCard
          title="Licences activées"
          value={stats.activatedLicenses}
          icon={KeyRound}
        />
      </div>

      {stats.expiringSoon.length > 0 && (
        <div className="rounded-2xl border border-amber-500/30 bg-amber-500/10 p-4 flex gap-3">
          <AlertTriangle className="w-5 h-5 text-amber-700 shrink-0" />
          <div>
            <p className="font-semibold text-amber-900">
              {stats.expiringSoon.length} abonnement(s) expirent sous 7 jours
            </p>
            <ul className="mt-2 text-sm text-amber-800/90 space-y-1">
              {stats.expiringSoon.slice(0, 5).map((s) => (
                <li key={s.id}>
                  Tenant {s.tenant_id.slice(0, 8)}… — {formatDate(s.expires_at)}
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}

      <div className="grid lg:grid-cols-2 gap-6">
        <section className="bg-card border border-border rounded-2xl p-5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-bold">Dernières entreprises</h3>
            <Link to="/tenants" className="text-sm text-primary font-semibold">
              Voir tout
            </Link>
          </div>
          <ul className="space-y-3">
            {tenants.slice(0, 6).map((t) => (
              <li key={t.id} className="flex items-center justify-between gap-2 text-sm">
                <Link to={`/tenants/${t.id}`} className="font-semibold hover:text-primary truncate">
                  {t.name}
                </Link>
                <Badge
                  label={t.is_active === false ? "Suspendu" : "Actif"}
                  tone={t.is_active === false ? "suspended" : "active"}
                />
              </li>
            ))}
            {tenants.length === 0 && (
              <p className="text-muted-foreground text-sm">Aucune entreprise.</p>
            )}
          </ul>
        </section>

        <section className="bg-card border border-border rounded-2xl p-5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-bold">Licences récentes</h3>
            <Link to="/licenses" className="text-sm text-primary font-semibold">
              Voir tout
            </Link>
          </div>
          <ul className="space-y-3 text-sm">
            {licenses.map((l) => (
              <li key={l.id} className="flex justify-between gap-2">
                <span className="font-mono text-xs">{l.license_key_masked}</span>
                <span className="text-muted-foreground shrink-0">
                  {l.activated_at ? "Activée" : "En attente"}
                </span>
              </li>
            ))}
            {licenses.length === 0 && (
              <p className="text-muted-foreground">Aucune licence générée.</p>
            )}
          </ul>
        </section>
      </div>
    </div>
  );
}
