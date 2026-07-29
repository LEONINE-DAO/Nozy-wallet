/** Minimal Vercel serverless types — avoid depending on `@vercel/node` (heavy vulnerable transitive tree). */
type VercelRequest = {
  method?: string;
};

type VercelResponse = {
  setHeader: (name: string, value: string) => void;
  status: (code: number) => VercelResponse;
  json: (body: unknown) => void;
  end: () => void;
};

const ACTIVATION_HEIGHT = 3_428_143;

type RpcResult = {
  result?: unknown;
  error?: { message?: string; code?: number };
};

async function zebraRpc(url: string, method: string, params: unknown[] = []) {
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: method,
      method,
      params,
    }),
  });
  if (!res.ok) {
    throw new Error(`RPC HTTP ${res.status}`);
  }
  const body = (await res.json()) as RpcResult;
  if (body.error) {
    throw new Error(body.error.message || `RPC error ${body.error.code ?? ""}`);
  }
  return body.result;
}

function poolChainValueZec(
  info: Record<string, unknown>,
  poolId: string
): number | null {
  const pools = info.valuePools;
  if (!Array.isArray(pools)) return null;
  for (const pool of pools) {
    if (!pool || typeof pool !== "object") continue;
    const p = pool as Record<string, unknown>;
    if (p.id === poolId) {
      const zec = p.chainValue;
      return typeof zec === "number" && Number.isFinite(zec) ? zec : null;
    }
  }
  return null;
}

export default async function handler(req: VercelRequest, res: VercelResponse) {
  res.setHeader("Cache-Control", "s-maxage=45, stale-while-revalidate=120");
  res.setHeader("Access-Control-Allow-Origin", "*");

  if (req.method === "OPTIONS") {
    res.status(204).end();
    return;
  }

  const rpcUrl = (process.env.ZEBRA_RPC_URL || "").trim();
  if (!rpcUrl) {
    res.status(200).json({
      tip: null,
      activationHeight: ACTIVATION_HEIGHT,
      orchardZec: null,
      ironwoodZec: null,
      migratedPct: null,
      available: false,
      fetchedAt: new Date().toISOString(),
      error: "ZEBRA_RPC_URL is not configured on this deploy",
    });
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
      throw new Error("Invalid getblockcount result");
    }

    const infoRaw = await zebraRpc(rpcUrl, "getblockchaininfo");
    if (!infoRaw || typeof infoRaw !== "object") {
      throw new Error("Invalid getblockchaininfo result");
    }
    const info = infoRaw as Record<string, unknown>;
    const orchardZec = poolChainValueZec(info, "orchard");
    const ironwoodZec = poolChainValueZec(info, "ironwood");
    const total =
      (orchardZec ?? 0) + (ironwoodZec ?? 0);
    const migratedPct =
      orchardZec != null && ironwoodZec != null && total > 0
        ? (ironwoodZec / total) * 100
        : null;

    res.status(200).json({
      tip,
      activationHeight: ACTIVATION_HEIGHT,
      orchardZec,
      ironwoodZec,
      migratedPct,
      available: true,
      fetchedAt: new Date().toISOString(),
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : "RPC failed";
    res.status(200).json({
      tip: null,
      activationHeight: ACTIVATION_HEIGHT,
      orchardZec: null,
      ironwoodZec: null,
      migratedPct: null,
      available: false,
      fetchedAt: new Date().toISOString(),
      error: message,
    });
  }
}
