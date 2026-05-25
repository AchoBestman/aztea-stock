export interface R2UploadStatus {
  configured: boolean;
  missing: string[];
  publicUrl: string;
}

export async function fetchR2UploadStatus(): Promise<R2UploadStatus> {
  const res = await fetch("/__r2/status");
  if (!res.ok) {
    return { configured: false, missing: [], publicUrl: "" };
  }
  return res.json() as Promise<R2UploadStatus>;
}
