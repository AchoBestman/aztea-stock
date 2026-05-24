/** R2 public base URL (no trailing slash). */
export function getR2PublicBaseUrl(): string {
  return (import.meta.env.VITE_R2_PUBLIC_URL || "").replace(/\/$/, "");
}

export function publicUrlForKey(key: string): string {
  const base = getR2PublicBaseUrl();
  const path = key.replace(/^\//, "");
  return base ? `${base}/${path}` : path;
}
