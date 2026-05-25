/** Variables d'environnement requises pour l'upload R2 (middleware Vite, côté Node). */

const REQUIRED = [
  "R2_ACCOUNT_ID",
  "R2_ACCESS_KEY_ID",
  "R2_SECRET_ACCESS_KEY",
  "R2_BUCKET_NAME",
] as const;

export function getMissingR2EnvVars(): string[] {
  return REQUIRED.filter((name) => !(process.env[name] || "").trim());
}

export function isR2UploadConfigured(): boolean {
  return getMissingR2EnvVars().length === 0;
}

export function getR2PublicBaseUrl(): string {
  const url =
    process.env.R2_PUBLIC_URL ||
    process.env.VITE_R2_PUBLIC_URL ||
    "";
  return url.replace(/\/$/, "");
}

export function assertR2UploadConfigured(): void {
  const missing = getMissingR2EnvVars();
  if (missing.length === 0) return;
  throw new Error(
    `Configuration R2 incomplète (${missing.join(", ")} manquant). ` +
      "Copiez admin/.env.example vers admin/.env et renseignez vos identifiants Cloudflare R2, puis redémarrez pnpm dev."
  );
}
