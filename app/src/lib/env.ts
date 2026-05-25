const STORAGE_API_URL = "aztea_api_base_url";

/** Default API root — must end with /api/v1 */
export function getDefaultApiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL || "http://localhost:8080/api/v1";
}

export function getApiBaseUrl(): string {
  return localStorage.getItem(STORAGE_API_URL) || getDefaultApiBaseUrl();
}

export function setApiBaseUrl(url: string): void {
  localStorage.setItem(STORAGE_API_URL, url.replace(/\/$/, ""));
}
