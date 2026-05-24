import {
  S3Client,
  PutObjectCommand,
  GetObjectCommand,
  DeleteObjectCommand,
} from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

function env(name: string): string {
  return process.env[name] || "";
}

const accountId = env("R2_ACCOUNT_ID");
const accessKey = env("R2_ACCESS_KEY_ID");
const secretKey = env("R2_SECRET_ACCESS_KEY");

export const r2Client = new S3Client({
  region: "auto",
  endpoint: accountId
    ? `https://${accountId}.r2.cloudflarestorage.com`
    : undefined,
  credentials:
    accessKey && secretKey
      ? { accessKeyId: accessKey, secretAccessKey: secretKey }
      : undefined,
});

const BUCKET = env("R2_BUCKET_NAME");

export async function uploadToR2(
  key: string,
  body: Buffer | Uint8Array,
  contentType: string,
  metadata?: Record<string, string>
): Promise<void> {
  if (!BUCKET) throw new Error("R2_BUCKET_NAME manquant");
  await r2Client.send(
    new PutObjectCommand({
      Bucket: BUCKET,
      Key: key,
      Body: body,
      ContentType: contentType,
      Metadata: metadata,
    })
  );
}

export async function getPresignedDownloadUrl(
  key: string,
  expiresIn = 3600
): Promise<string> {
  const command = new GetObjectCommand({ Bucket: BUCKET, Key: key });
  return getSignedUrl(r2Client, command, { expiresIn });
}

export async function deleteFromR2(key: string): Promise<void> {
  await r2Client.send(
    new DeleteObjectCommand({ Bucket: BUCKET, Key: key })
  );
}
