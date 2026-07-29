import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Vercel root deploy uses `/`. GitHub Pages project site uses `/Nozy-wallet/`.
const base =
  process.env.VERCEL === "1" || process.env.VITE_BASE === "/"
    ? "/"
    : process.env.VITE_BASE || "/Nozy-wallet/";

const ACTIVATION_HEIGHT = 3_428_143;

async function zebraRpc(url: string, method: string) {
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: method,
      method,
      params: [],
    }),
  });
  if (!res.ok) throw new Error(`RPC HTTP ${res.status}`);
  const body = (await res.json()) as {
    result?: unknown;
    error?: { message?: string };
  };
  if (body.error) throw new Error(body.error.message || "RPC error");
  return body.result;
}

function poolZec(info: Record<string, unknown>, id: string): number | null {
  const pools = info.valuePools;
  if (!Array.isArray(pools)) return null;
  for (const pool of pools) {
    if (!pool || typeof pool !== "object") continue;
    const p = pool as Record<string, unknown>;
    if (p.id === id && typeof p.chainValue === "number") return p.chainValue;
  }
  return null;
}

/** Local-dev stand-in for Vercel `/api/ironwood-stats` when `ZEBRA_RPC_URL` is set. */
function ironwoodStatsDevApi(): Plugin {
  return {
    name: "ironwood-stats-dev-api",
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        const url = req.url?.split("?")[0] ?? "";
        if (url !== "/api/ironwood-stats") {
          next();
          return;
        }
        res.setHeader("Content-Type", "application/json");
        res.setHeader("Cache-Control", "no-store");
        const rpcUrl = (process.env.ZEBRA_RPC_URL || "").trim();
        if (!rpcUrl) {
          res.end(
            JSON.stringify({
              tip: null,
              activationHeight: ACTIVATION_HEIGHT,
              orchardZec: null,
              ironwoodZec: null,
              migratedPct: null,
              available: false,
              fetchedAt: new Date().toISOString(),
              error:
                "Set ZEBRA_RPC_URL for local live stats (e.g. http://172.x.x.x:8232)",
            })
          );
          return;
        }
        try {
          const tipRaw = await zebraRpc(rpcUrl, "getblockcount");
          const tip =
            typeof tipRaw === "number"
              ? tipRaw
              : typeof tipRaw === "string"
                ? Number(tipRaw)
                : null;
          if (tip == null || !Number.isFinite(tip)) {
            throw new Error("Invalid getblockcount");
          }
          const infoRaw = await zebraRpc(rpcUrl, "getblockchaininfo");
          if (!infoRaw || typeof infoRaw !== "object") {
            throw new Error("Invalid getblockchaininfo");
          }
          const info = infoRaw as Record<string, unknown>;
          const orchardZec = poolZec(info, "orchard");
          const ironwoodZec = poolZec(info, "ironwood");
          const total = (orchardZec ?? 0) + (ironwoodZec ?? 0);
          const migratedPct =
            orchardZec != null && ironwoodZec != null && total > 0
              ? (ironwoodZec / total) * 100
              : null;
          res.end(
            JSON.stringify({
              tip,
              activationHeight: ACTIVATION_HEIGHT,
              orchardZec,
              ironwoodZec,
              migratedPct,
              available: true,
              fetchedAt: new Date().toISOString(),
            })
          );
        } catch (err) {
          res.end(
            JSON.stringify({
              tip: null,
              activationHeight: ACTIVATION_HEIGHT,
              orchardZec: null,
              ironwoodZec: null,
              migratedPct: null,
              available: false,
              fetchedAt: new Date().toISOString(),
              error: err instanceof Error ? err.message : "RPC failed",
            })
          );
        }
      });
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), ironwoodStatsDevApi()],
  base,
  resolve: {
    dedupe: ["react", "react-dom"],
  },
});
