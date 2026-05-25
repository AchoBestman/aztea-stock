import type { Plugin } from "vite";
import type { IncomingMessage, ServerResponse } from "node:http";
import { loadEnv } from "vite";
import Busboy from "busboy";
import { uploadToR2 } from "./src/lib/r2/client";
import {
  getMissingR2EnvVars,
  getR2PublicBaseUrl,
  isR2UploadConfigured,
} from "./src/lib/r2/serverConfig";

async function handleUpload(
  req: IncomingMessage,
  res: ServerResponse
): Promise<void> {
  const contentType = req.headers["content-type"] || "";
  if (!contentType.includes("multipart/form-data")) {
    res.statusCode = 400;
    res.end(JSON.stringify({ message: "multipart/form-data requis" }));
    return;
  }

  await new Promise<void>((resolve, reject) => {
    const bb = Busboy({ headers: req.headers });
    let key = "";
    let fileBuffer: Buffer | null = null;
    let fileMime = "application/octet-stream";

    bb.on("field", (name, val) => {
      if (name === "key") key = val;
    });

    bb.on("file", (_name, stream, info) => {
      fileMime = info.mimeType || fileMime;
      const chunks: Buffer[] = [];
      stream.on("data", (d: Buffer) => chunks.push(d));
      stream.on("end", () => {
        fileBuffer = Buffer.concat(chunks);
      });
    });

    bb.on("error", reject);
    bb.on("close", async () => {
      try {
        if (!key || !fileBuffer) {
          res.statusCode = 400;
          res.end(JSON.stringify({ message: "key et file requis" }));
          resolve();
          return;
        }
        await uploadToR2(key, fileBuffer, fileMime);
        const base = getR2PublicBaseUrl();
        const url = base ? `${base}/${key}` : key;
        res.setHeader("Content-Type", "application/json");
        res.end(JSON.stringify({ key, url }));
        resolve();
      } catch (e) {
        reject(e);
      }
    });

    req.pipe(bb);
  });
}

function applyEnv(mode: string) {
  const env = loadEnv(mode, process.cwd(), "");
  Object.assign(process.env, env);
}

function handleStatus(res: ServerResponse): void {
  res.setHeader("Content-Type", "application/json");
  res.end(
    JSON.stringify({
      configured: isR2UploadConfigured(),
      missing: getMissingR2EnvVars(),
      publicUrl: getR2PublicBaseUrl(),
    })
  );
}

function r2Middleware(
  req: IncomingMessage,
  res: ServerResponse,
  next: () => void
): void {
  if (req.url === "/__r2/status" && req.method === "GET") {
    handleStatus(res);
    return;
  }
  if (!req.url?.startsWith("/__r2/upload")) {
    next();
    return;
  }
  if (req.method !== "POST") {
    res.statusCode = 405;
    res.end();
    return;
  }
  void handleUpload(req, res).catch((e) => {
    res.statusCode = 500;
    res.end(
      JSON.stringify({
        message: e instanceof Error ? e.message : "Erreur R2",
      })
    );
  });
}

export function r2UploadPlugin(): Plugin {
  return {
    name: "aztea-r2-upload",
    configureServer(server) {
      applyEnv(server.config.mode);
      server.middlewares.use(r2Middleware);
    },
    configurePreviewServer(server) {
      applyEnv("production");
      server.middlewares.use(r2Middleware);
    },
  };
}
