/** Zebrad JSON-RPC presets + public lightwalletd (zec.rocks-style) for the extension. */

export const DEFAULT_RPC = "http://127.0.0.1:8232";
export const DEFAULT_TESTNET_RPC = "http://127.0.0.1:18232";

/** Nozy public compact-sync LWD — keys stay on the device (same model as zec.rocks). */
export const PUBLIC_LWD_URL = "https://lwd.nozywallet.org:443";

/** Org status probe (HTTPS JSON) — confirms public LWD / Zebrad health without gRPC from the browser. */
export const PUBLIC_NODE_STATUS_URL = "https://nozywallet.org/status.json";

export type NodeSetupMode = "auto" | "local" | "wsl" | "remote" | "public";

/** Load-page chips: local discovery vs NozyWallet compact sync. */
export const NODE_SETUP_MODES: Array<{ id: NodeSetupMode; label: string }> = [
  { id: "local", label: "Local node" },
  { id: "public", label: "NozyWallet" }
];

export function isPublicLwdUrl(url: string): boolean {
  const u = url.trim().toLowerCase();
  return u.includes("lwd.nozywallet.org") || /(^|\/\/)(mainnet\.)?zec\.rocks\b/.test(u);
}

export function isWslZebradUrl(url: string): boolean {
  return /^https?:\/\/172\.(1[6-9]|2\d|3[0-1])\.\d+\.\d+:8232\/?$/i.test(url.trim());
}

export function rpcPresetId(url: string): string {
  if (isPublicLwdUrl(url)) return "public-lwd";
  if (isWslZebradUrl(url)) return "mainnet-wsl";
  if (url === DEFAULT_RPC) return "mainnet-local";
  if (url === DEFAULT_TESTNET_RPC) return "testnet-local";
  return "custom";
}

export function setupHelp(mode: NodeSetupMode): string[] {
  switch (mode) {
    case "public":
      return [
        "NozyWallet sync uses lwd.nozywallet.org for compact blocks.",
        "Your seed stays in this extension — keys never leave this device.",
        "Click Connect when you are ready."
      ];
    case "local":
    default:
      return [
        "Start Zebrad or Zakura on this computer (or Crosslink).",
        "Click Find node — we look for a local JSON-RPC endpoint.",
        "If nothing is found, pick NozyWallet instead."
      ];
  }
}

export function connectFailureHint(mode: NodeSetupMode): string {
  if (mode === "public") {
    return "NozyWallet sync could not reach lwd.nozywallet.org. Check https://nozywallet.org/status.json and try again.";
  }
  return "Is a local node running? Or pick NozyWallet to sync without one.";
}

export type PublicNodeStatus = {
  ok: boolean;
  blockCount: number | null;
  lwdOk: boolean;
  zebraOk: boolean;
  message: string;
};

/** Probe nozywallet.org/status.json (public LWD health). */
export async function probePublicNodeStatus(
  timeoutMs = 6000
): Promise<PublicNodeStatus> {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const resp = await fetch(PUBLIC_NODE_STATUS_URL, {
      cache: "no-store",
      signal: ctrl.signal
    });
    if (!resp.ok) {
      return {
        ok: false,
        blockCount: null,
        lwdOk: false,
        zebraOk: false,
        message: `Public node status HTTP ${resp.status}`
      };
    }
    const body = (await resp.json()) as {
      lightwalletd?: { ok?: boolean; height?: number };
      zebra?: { ok?: boolean; blocks?: number };
    };
    const lwdOk = Boolean(body?.lightwalletd?.ok);
    const zebraOk = Boolean(body?.zebra?.ok);
    const heightRaw = body?.lightwalletd?.height ?? body?.zebra?.blocks;
    const blockCount =
      typeof heightRaw === "number" && Number.isFinite(heightRaw) && heightRaw >= 0
        ? Math.floor(heightRaw)
        : null;
    if (!lwdOk) {
      return {
        ok: false,
        blockCount,
        lwdOk,
        zebraOk,
        message: "Public lightwalletd is not ready yet. Try again later or use a local node."
      };
    }
    return {
      ok: true,
      blockCount,
      lwdOk,
      zebraOk,
      message:
        blockCount != null
          ? `NozyWallet ready — ${blockCount.toLocaleString()} blocks (lwd.nozywallet.org)`
          : "NozyWallet ready (lwd.nozywallet.org)"
    };
  } catch (e) {
    return {
      ok: false,
      blockCount: null,
      lwdOk: false,
      zebraOk: false,
      message: (e as Error)?.message || "Could not reach public node status"
    };
  } finally {
    clearTimeout(timer);
  }
}
