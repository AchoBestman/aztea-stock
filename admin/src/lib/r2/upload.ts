import { avatarObjectKey, tenantSlug } from "./slug";
import { publicUrlForKey } from "./config";

const R2_UPLOAD_PATH = "/__r2/upload";

export interface UploadResult {
  key: string;
  url: string;
}

/**
 * Upload avatar via le middleware Vite (dev/preview) qui utilise lib/r2/client.ts côté Node.
 */
export async function uploadTenantAvatar(
  tenantName: string,
  file: File,
  onProgress?: (percent: number) => void
): Promise<UploadResult> {
  const slug = tenantSlug(tenantName);
  const key = avatarObjectKey(slug, file.name);

  const form = new FormData();
  form.append("file", file);
  form.append("key", key);

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();

    xhr.upload.addEventListener("progress", (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress(Math.round((e.loaded / e.total) * 100));
      }
    });

    xhr.addEventListener("load", () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          const data = JSON.parse(xhr.responseText) as { key: string; url?: string };
          resolve({ key: data.key, url: data.url ?? publicUrlForKey(data.key) });
        } catch {
          reject(new Error("Réponse invalide de R2"));
        }
      } else {
        try {
          const err = JSON.parse(xhr.responseText) as { message?: string };
          reject(new Error(err.message || "Échec de l'upload vers R2"));
        } catch {
          reject(new Error("Échec de l'upload vers R2"));
        }
      }
    });

    xhr.addEventListener("error", () => reject(new Error("Échec de l'upload vers R2")));
    xhr.addEventListener("abort", () => reject(new Error("Upload annulé")));

    xhr.open("POST", R2_UPLOAD_PATH);
    xhr.send(form);
  });
}
