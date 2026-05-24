import type { Plugin } from "vite";
import type { IncomingMessage, ServerResponse } from "node:http";
import { loadEnv } from "vite";
import Busboy from "busboy";
import { uploadToR2 } from "./src/lib/r2/client";

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
        const base = (process.env.R2_PUBLIC_URL || "").replace(/\/$/, "");
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

export function r2UploadPlugin(): Plugin {
  return {
    name: "aztea-r2-upload",
    configureServer(server) {
      const env = loadEnv(server.config.mode, process.cwd(), "");
      Object.assign(process.env, env);

      server.middlewares.use(async (req, res, next) => {
        if (!req.url?.startsWith("/__r2/upload")) return next();
        if (req.method !== "POST") {
          res.statusCode = 405;
          res.end();
          return;
        }
        try {
          await handleUpload(req, res);
        } catch (e) {
          res.statusCode = 500;
          res.end(
            JSON.stringify({
              message: e instanceof Error ? e.message : "Erreur R2",
            })
          );
        }
      });
    },
    configurePreviewServer(server) {
      const env = loadEnv("production", process.cwd(), "");
      Object.assign(process.env, env);
      server.middlewares.use(async (req, res, next) => {
        if (!req.url?.startsWith("/__r2/upload")) return next();
        if (req.method !== "POST") {
          res.statusCode = 405;
          res.end();
          return;
        }
        try {
          await handleUpload(req, res);
        } catch (e) {
          res.statusCode = 500;
          res.end(
            JSON.stringify({
              message: e instanceof Error ? e.message : "Erreur R2",
            })
          );
        }
      });
    },
  };
}
