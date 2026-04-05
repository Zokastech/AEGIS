/**
 * AEGIS — zokastech.fr — Apache 2.0 / MIT
 *
 * Express : appelle la passerelle AEGIS (`POST /v1/anonymize`) — même moteur Rust que le reste de la stack.
 *
 * Variables : `AEGIS_BASE_URL` (défaut http://127.0.0.1:8080), `AEGIS_API_KEY` + optionnel `AEGIS_API_KEY_HEADER`,
 * `PORT` (défaut 3000).
 *
 * Lancer : npm start
 */

import https from "node:https";
import axios from "axios";
import express from "express";

const BASE = (process.env.AEGIS_BASE_URL || "http://127.0.0.1:8080").replace(/\/$/, "");
const PORT = Number(process.env.PORT || "3000");
const apiKey = (process.env.AEGIS_API_KEY || "").trim();
const apiKeyHeader = (process.env.AEGIS_API_KEY_HEADER || "X-API-Key").trim() || "X-API-Key";

const tlsInsecure =
  BASE.startsWith("https://") &&
  (process.env.AEGIS_TLS_SKIP_VERIFY === "1" || process.env.NODE_TLS_REJECT_UNAUTHORIZED === "0");

const gateway = axios.create({
  baseURL: BASE,
  timeout: 60_000,
  headers: {
    "Content-Type": "application/json",
    ...(apiKey ? { [apiKeyHeader]: apiKey } : {}),
  },
  ...(tlsInsecure ? { httpsAgent: new https.Agent({ rejectUnauthorized: false }) } : {}),
});

const anonConfig = JSON.stringify({
  operators_by_entity: {
    EMAIL: { operator_type: "mask", params: { keep_last: "4", mask_char: "*" } },
    PHONE: { operator_type: "redact", params: { replacement: "[PHONE]" } },
  },
});

function anonymizedTextFromPayload(data) {
  if (data == null || typeof data !== "object") return String(data);
  const r = data.result !== undefined ? data.result : data;
  const inner = typeof r === "string" ? JSON.parse(r) : r;
  const a = inner?.anonymized ?? inner;
  if (a && typeof a === "object" && "text" in a) return String(a.text);
  return typeof a === "string" ? a : JSON.stringify(a);
}

async function anonymizeText(text) {
  if (!text || typeof text !== "string") return text;
  const { data } = await gateway.post("/v1/anonymize", {
    text,
    config_json: anonConfig,
  });
  return anonymizedTextFromPayload(data);
}

const app = express();
app.use(express.json({ limit: "1mb" }));

app.get("/health", async (_req, res) => {
  const paths = ["/livez", "/health/live"];
  for (const p of paths) {
    try {
      const r = await gateway.get(p, { validateStatus: () => true, timeout: 5000 });
      if (r.status === 200 || r.status === 204) {
        res.json({ ok: true, gateway: BASE, probe: p });
        return;
      }
    } catch {
      /* try next path */
    }
  }
  res.status(503).json({ ok: false, gateway: BASE, error: "no live probe OK" });
});

app.post("/api/note", async (req, res) => {
  const text = typeof req.body?.text === "string" ? req.body.text : "";
  try {
    const sanitized = await anonymizeText(text);
    res.json({
      ok: true,
      text_length: text.length,
      text_sanitized: sanitized,
      source: "aegis-gateway",
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    console.error("[aegis] anonymize failed — gateway up?", msg);
    res.status(502).json({
      ok: false,
      error: msg,
      hint: "Vérifiez AEGIS_BASE_URL, TLS/port (8443 HTTPS vs 8080 HTTP), et AEGIS_API_KEY si l’auth est activée.",
    });
  }
});

app.listen(PORT, () => {
  console.log(`Express → AEGIS gateway on http://127.0.0.1:${PORT} (gateway ${BASE}${apiKey ? ", API key set" : ""})`);
  console.log(`Try: curl -s -X POST http://127.0.0.1:${PORT}/api/note -H "Content-Type: application/json" -d '{"text":"Mail: demo@example.com"}'`);
});
