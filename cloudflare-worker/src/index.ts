export interface EmailJob {
  id: string;
  tenant_id: string;
  to: string;
  subject: string;
  html: string;
  attempts: number;
  max_attempts: number;
  scheduled_at: string;
  created_at: string;
}

export interface Env {
  NEXT_PUBLIC_APP_URL: string; // The base URL of your Rust backend API (e.g. https://api.aztea.com)
  CRON_SECRET: string;        // Shared secret token configured in both Rust .env and Wrangler
}

export default {
  // ── 1. Queue Consumer ──────────────────────────────────────────────────────
  async queue(batch: MessageBatch<any>, env: Env): Promise<void> {
    for (const message of batch.messages) {
      await this.processMessage(message, env);
    }
  },

  // ── 2. Local/HTTP Trigger Handler ──────────────────────────────────────────
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "POST") {
      try {
        const body = await request.json();
        await this.processMessage({ body, ack: () => { }, retry: () => { } } as any, env);
        return new Response("OK - Message enqueued locally", { status: 200 });
      } catch (err: any) {
        return new Response("Error: " + err.message, { status: 400 });
      }
    }
    return new Response("Aztea Stock Email Worker is active. Use POST to test.", { status: 200 });
  },

  // ── 3. Message Processor ───────────────────────────────────────────────────
  async processMessage(message: Message<any>, env: Env): Promise<void> {
    try {
      const job: EmailJob = typeof message.body === 'string'
        ? JSON.parse(message.body)
        : message.body;

      console.log(`🚀 [CF Worker] Processing → ${job.to} : ${job.subject}`);

      // Call the Rust API's internal callback endpoint
      const res = await fetch(`${env.NEXT_PUBLIC_APP_URL}/api/v1/internal/send-email`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-internal-secret": env.CRON_SECRET,
        },
        body: JSON.stringify(job),
      });

      if (res.ok) {
        console.log(`  ✅ Successfully sent email to ${job.to} (Tenant: ${job.tenant_id})!`);
        message.ack();
      } else {
        const err = await res.text();
        console.warn(`  ⚠️ Rust API error (${res.status}): ${err}`);
        message.retry();
      }
    } catch (error) {
      console.error("❌ CF Worker unexpected error:", error);
      message.retry();
    }
  }
};
