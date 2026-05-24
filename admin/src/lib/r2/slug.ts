/** Dossier R2 pour un tenant : slug du nom commercial */
export function tenantSlug(name: string): string {
  const base = name
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return base || "tenant";
}

export function avatarObjectKey(slug: string, filename: string): string {
  const ext = filename.includes(".")
    ? filename.split(".").pop()!.toLowerCase()
    : "jpg";
  const safeExt = ext.replace(/[^a-z0-9]/g, "") || "jpg";
  return `${slug}/avatar.${safeExt}`;
}
