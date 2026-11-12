import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createDeskWallet, deskSnapshot, draftInvoice, invoiceStatus, unlockWallet } from "./companion.mjs";
import { ask } from "./near-agent.mjs";
import { shieldStatus, startShield, walletSetup } from "./setup.mjs";
import { fetchGatewayAttestation } from "./tee.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const publicDir = path.join(here, "..", "public");

function loadLocalEnv() {
  try {
    const text = fs.readFileSync(path.join(here, "..", ".env"), "utf8");
    for (const line of text.split(/\r?\n/)) {
      const match = line.match(/^([A-Z0-9_]+)=(.*)$/);
      if (!match || process.env[match[1]]) continue;
      process.env[match[1]] = match[2].trim();
    }
  } catch {
    // A missing local env file leaves the key unset.
  }
}

loadLocalEnv();
const port = Number(process.env.PORT || 4173);

const types = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
};

async function readJson(req) {
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  const text = Buffer.concat(chunks).toString("utf8");
  if (!text) return {};
  return JSON.parse(text);
}

function send(res, status, body, type) {
  res.writeHead(status, {
    "content-type": type,
    "cache-control": "no-store",
  });
  res.end(body);
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", "http://127.0.0.1");

  const invoiceMatch = url.pathname.match(/^\/api\/invoices\/([A-Za-z0-9_-]+)$/);
  if (req.method === "GET" && invoiceMatch) {
    send(res, 200, JSON.stringify(await invoiceStatus(invoiceMatch[1])), "application/json");
    return;
  }

  if (req.method === "GET" && url.pathname === "/api/desk") {
    send(res, 200, JSON.stringify(await deskSnapshot()), "application/json");
    return;
  }

  if (req.method === "GET" && url.pathname === "/api/tee") {
    send(res, 200, JSON.stringify(await fetchGatewayAttestation({ apiKey: process.env.NEAR_AI_API_KEY })), "application/json");
    return;
  }

  if (req.method === "GET" && url.pathname === "/api/setup") {
    send(res, 200, JSON.stringify(await walletSetup()), "application/json");
    return;
  }

  if (req.method === "GET" && url.pathname === "/api/shield") {
    send(res, 200, JSON.stringify({ ok: true, shield: shieldStatus() }), "application/json");
    return;
  }

  if (req.method === "POST" && url.pathname === "/api/shield") {
    let input = {};
    try {
      input = await readJson(req);
    } catch {
      send(res, 400, JSON.stringify({ ok: false, reason: "invalid JSON" }), "application/json");
      return;
    }
    if (!input.confirmed) {
      send(res, 200, JSON.stringify({ ok: false, needsConfirm: true, shield: shieldStatus() }), "application/json");
      return;
    }
    send(res, 200, JSON.stringify(startShield()), "application/json");
    return;
  }

  if (req.method === "POST" && (url.pathname === "/api/act" || url.pathname === "/api/wallet/unlock" || url.pathname === "/api/wallet/create")) {
    let input = {};
    try {
      input = await readJson(req);
    } catch {
      send(res, 400, JSON.stringify({ ok: false, reason: "invalid JSON" }), "application/json");
      return;
    }
    if (url.pathname === "/api/wallet/create") {
      const password = String(input.password ?? "");
      const view = await createDeskWallet(password);
      send(res, 200, JSON.stringify(view), "application/json");
      return;
    }
    if (url.pathname === "/api/wallet/unlock") {
      const password = String(input.password ?? "");
      const view = await unlockWallet(password);
      send(res, 200, JSON.stringify(view), "application/json");
      return;
    }
    const { password, ...rest } = input;
    const decision = await ask(rest);
    if (decision.handoff === "invoice") {
      send(
        res,
        200,
        JSON.stringify(await draftInvoice({ amount: decision.amount, memo: decision.memo, password })),
        "application/json",
      );
      return;
    }
    send(res, 200, JSON.stringify(decision), "application/json");
    return;
  }

  if (req.method !== "GET" && req.method !== "HEAD") {
    send(res, 405, "Method not allowed", "text/plain; charset=utf-8");
    return;
  }

  const rel = url.pathname === "/" ? "index.html" : url.pathname.replace(/^\/+/, "");
  const root = path.resolve(publicDir);
  const file = path.resolve(root, rel);
  if (file !== root && !file.startsWith(root + path.sep)) {
    send(res, 404, "Not found", "text/plain; charset=utf-8");
    return;
  }
  fs.readFile(file, (error, data) => {
    if (error) {
      send(res, 404, "Not found", "text/plain; charset=utf-8");
      return;
    }
    send(res, 200, data, types[path.extname(file)] ?? "application/octet-stream");
  });
});

server.listen(port, "127.0.0.1", () => {
  console.log(`Nozy Agent GUI  http://127.0.0.1:${port}`);
});
