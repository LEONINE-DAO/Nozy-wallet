/** Zebrad JSON-RPC presets + public lightwalletd (zec.rocks-style) for the extension. */

export const DEFAULT_RPC = "http://127.0.0.1:8232";
export const DEFAULT_TESTNET_RPC = "http://127.0.0.1:18232";

/** Nozy public compact-sync LWD — keys stay on the device (same model as zec.rocks). */
export const PUBLIC_LWD_URL = "https://lwd.nozywallet.org:443";

/** Org status probe (HTTPS JSON) — confirms public LWD / Zebrad health without gRPC from the browser. */
export const PUBLIC_NODE_STATUS_URL = "https://nozywallet.org/status.json";

export type NodeSetupMode = "auto" | "local" | "wsl" | "remote" | "public";

export const NODE_SETUP_MODES: Array<{ id: NodeSetupMode; label: string }> = [
  { id: "auto", label: "Find automatically" },
  { id: "local", label: "This PC" },
  { id: "wsl", label: "WSL / Linux VM" },
  { id: "public", label: "Public sync" },
  { id: "remote", label: "Remote VPS" }
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
    case "local":
      return [
        "Start Zebrad on this computer (JSON-RPC port 8232).",
        "Click Connect — we use http://127.0.0.1:8232."
      ];
    case "wsl":
      return [
        "Start Zebrad inside WSL (Ubuntu): zebrad start or your usual script.",
        "Click Find my node — Chrome on Windows cannot use 127.0.0.1 for WSL.",
        "We auto-detect the WSL IP (http://172.x.x.x:8232)."
      ];
    case "public":
      return [
        "Connect to Nozy’s public lightwalletd (lwd.nozywallet.org) — same idea as zec.rocks.",
        "Your seed stays in this extension. The public node only serves compact blocks.",
        "In-extension Orchard scan/send still prefer a local Zebrad when you have one."
      ];
    case "remote":
      return [
        "Your server must expose Zebrad JSON-RPC (HTTPS recommended).",
        "Paste the full URL below — e.g. https://your-node.example.com:443",
        "Click Connect. Ask your host for the RPC URL if unsure."
      ];
    default:
      return [
        "Start Zebrad (this PC, WSL, or VPS), or use Public sync if you have no node yet.",
        "Click Find my node — we try local ports, WSL IP, and Nozy Desktop config.",
        "If that fails, pick Public sync (zec.rocks-style) or Remote and follow the steps."
      ];
  }
}

export function connectFailureHint(mode: NodeSetupMode): string {
  if (mode === "public") {
    return "Public sync uses lwd.nozywallet.org. Check https://nozywallet.org/status.json if this keeps failing.";
  }
  if (mode === "wsl" || mode === "auto") {
    return "Still stuck? Pick Public sync (no local node), or in PowerShell run: wsl -d Ubuntu -- hostname -I — use http://<first-IP>:8232 under Remote VPS.";
  }
  if (mode === "remote") {
    return "Check the URL includes http:// or https:// and the port matches your node (8232 local, 443 on many VPS setups).";
  }
  return "Is Zebrad running? Local JSON-RPC needs enable_cookie_auth=false in zebrad.toml for browser access. Or choose Public sync.";
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
          ? `Public sync ready — ${blockCount.toLocaleString()} blocks (lwd.nozywallet.org)`
          : "Public sync ready (lwd.nozywallet.org)"
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
