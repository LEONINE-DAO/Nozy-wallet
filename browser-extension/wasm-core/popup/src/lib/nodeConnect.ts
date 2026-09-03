/** Zebrad JSON-RPC presets shown in the extension node-connect wizard. */

export const DEFAULT_RPC = "http://127.0.0.1:8232";
export const DEFAULT_TESTNET_RPC = "http://127.0.0.1:18232";

export type NodeSetupMode = "auto" | "local" | "wsl" | "remote";

export const NODE_SETUP_MODES: Array<{ id: NodeSetupMode; label: string }> = [
  { id: "auto", label: "Find automatically" },
  { id: "local", label: "This PC" },
  { id: "wsl", label: "WSL / Linux VM" },
  { id: "remote", label: "Remote VPS" }
];

export function isWslZebradUrl(url: string): boolean {
  return /^https?:\/\/172\.(1[6-9]|2\d|3[0-1])\.\d+\.\d+:8232\/?$/i.test(url.trim());
}

export function rpcPresetId(url: string): string {
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
    case "remote":
      return [
        "Your server must expose Zebrad JSON-RPC (HTTPS recommended).",
        "Paste the full URL below — e.g. https://your-node.example.com:443",
        "Click Connect. Ask your host for the RPC URL if unsure."
      ];
    default:
      return [
        "Start Zebrad (this PC, WSL, or VPS).",
        "Click Find my node — we try local ports, WSL IP, and Nozy Desktop config.",
        "If that fails, pick This PC, WSL, or Remote and follow the steps."
      ];
  }
}

export function connectFailureHint(mode: NodeSetupMode): string {
  if (mode === "wsl" || mode === "auto") {
    return "Still stuck? In PowerShell run: wsl -d Ubuntu -- hostname -I — use http://<first-IP>:8232 under Remote VPS.";
  }
  if (mode === "remote") {
    return "Check the URL includes http:// or https:// and the port matches your node (8232 local, 443 on many VPS setups).";
  }
  return "Is Zebrad running? Local JSON-RPC needs enable_cookie_auth=false in zebrad.toml for browser access.";
}
