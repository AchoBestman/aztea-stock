export function formatCurrency(amount: number | string, currency = "XAF"): string {
  const n = typeof amount === "string" ? parseFloat(amount) : amount;
  if (Number.isNaN(n)) return `— ${currency}`;
  return new Intl.NumberFormat("fr-FR", {
    style: "currency",
    currency,
    maximumFractionDigits: 0,
  }).format(n);
}

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return new Intl.DateTimeFormat("fr-FR", {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

export function formatDateShort(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return new Intl.DateTimeFormat("fr-FR", { dateStyle: "medium" }).format(new Date(iso));
  } catch {
    return iso;
  }
}

export const PLAN_PRESETS: Record<
  string,
  { label: string; price: number; currency: string; max_devices: number }
> = {
  starter: { label: "Starter", price: 15000, currency: "XAF", max_devices: 1 },
  pro: { label: "Pro", price: 35000, currency: "XAF", max_devices: 3 },
  enterprise: { label: "Enterprise", price: 75000, currency: "XAF", max_devices: 10 },
};

export const BUSINESS_TYPES = [
  { value: "pharmacy", label: "Pharmacie" },
  { value: "supermarket", label: "Supermarché" },
  { value: "both", label: "Les deux" },
] as const;

export const SUBSCRIPTION_STATUSES = [
  { value: "trial", label: "Essai" },
  { value: "production", label: "Production" },
  { value: "active", label: "Actif" },
  { value: "suspended", label: "Suspendu" },
  { value: "cancelled", label: "Annulé" },
] as const;
