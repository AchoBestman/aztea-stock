import { useEffect, useState } from "react";
import { fetchR2UploadStatus } from "../lib/r2/status";

/** Indique si le middleware Vite peut envoyer des fichiers vers R2 (null = chargement). */
export function useR2UploadAvailable(): boolean | null {
  const [available, setAvailable] = useState<boolean | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchR2UploadStatus()
      .then((s) => {
        if (!cancelled) setAvailable(s.configured);
      })
      .catch(() => {
        if (!cancelled) setAvailable(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return available;
}
