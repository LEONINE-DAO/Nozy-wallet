import Constants from "expo-constants";
import { Platform } from "react-native";

/** How the phone reaches nozywallet-api. */
export type ApiConnectionMode = "self" | "hosted";

/** How the API server reaches Zebrad (Network & Node settings). */
export type NodeConnectionMode = "local" | "public";

export const PUBLIC_NODE_RISK_TITLE = "Public infrastructure risks";

export const PUBLIC_NODE_RISK_POINTS = [
  "Your wallet seed and keys are stored on the API server you connect to — not only on this phone.",
  "The operator can see your IP address, when you sync, and when you send transactions.",
  "Nym smolmix on the server does not remove operator trust for hosted mobile wallets.",
  "Shielded ZEC hides amounts and addresses on-chain, but not who runs your API or node.",
  "For maximum privacy, use your own PC or VPS with a local Zebrad node, or hardware signing (Keystone).",
] as const;

function expoExtra(key: string): string | undefined {
  const v = Constants.expoConfig?.extra?.[key];
  return typeof v === "string" && v.trim() ? v.trim() : undefined;
}

/** Default API URL for emulator / home PC companion. */
export function defaultSelfHostedApiUrl(): string {
  const fromConfig = expoExtra("selfHostedApiUrl");
  if (fromConfig) return fromConfig;
  if (Platform.OS === "android") return "http://10.0.2.2:3000";
  return "http://localhost:3000";
}

/** @deprecated No Nozy-hosted companion. Kept so Fast Refresh does not crash old bundles. */
export function defaultHostedApiUrl(): string {
  return defaultSelfHostedApiUrl();
}

/**
 * Optional lightwalletd URL for on-device compact sync.
 * Public Nozy node — keys stay on the phone; this is not the companion API.
 */
/** Nozy public compact-sync server. Keys stay on the device. Same value in the extension. */
export const NOZY_PUBLIC_LWD_URL = "https://lwd.nozywallet.org:443";

export function defaultHostedLwdUrl(): string {
  return expoExtra("hostedLwdUrl") ?? NOZY_PUBLIC_LWD_URL;
}

/** Retired Nozy-operated companion hostname — never a product default. */
export const RETIRED_NOZY_HOSTED_API_HOST = "nozywallet.leoninedao.org";

export function isRetiredNozyHostedApiUrl(url: string): boolean {
  const host = hostFromUrl(url);
  return host === RETIRED_NOZY_HOSTED_API_HOST;
}

/**
 * Optional public Zebrad JSON-RPC for Network settings.
 * Empty by default — no Nozy Zebrad until funding. Operator / user sets a live URL.
 */
export function defaultPublicZebraUrl(): string {
  return expoExtra("hostedZebraUrl") ?? "";
}

export function defaultLocalZebraUrl(): string {
  return "http://127.0.0.1:8232";
}

function hostFromUrl(raw: string): string | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;
  try {
    const withScheme = /^https?:\/\//i.test(trimmed)
      ? trimmed
      : `http://${trimmed}`;
    return new URL(withScheme).hostname.toLowerCase();
  } catch {
    return null;
  }
}

function isLoopbackHost(host: string): boolean {
  return (
    host === "localhost" ||
    host === "127.0.0.1" ||
    host === "::1" ||
    host === "10.0.2.2"
  );
}

function isPrivateLanHost(host: string): boolean {
  if (isLoopbackHost(host)) return true;
  const v4 = host.match(/^(\d+)\.(\d+)\.(\d+)\.(\d+)$/);
  if (!v4) return false;
  const a = Number(v4[1]);
  const b = Number(v4[2]);
  if (a === 10) return true;
  if (a === 172 && b >= 16 && b <= 31) return true;
  if (a === 192 && b === 168) return true;
  return false;
}

/** True when the phone talks to a home/local companion API. */
export function isLocalApiUrl(url: string): boolean {
  const host = hostFromUrl(url);
  if (!host) return false;
  return isPrivateLanHost(host);
}

/** True when the API URL is a public HTTPS host (not LAN / emulator). */
export function isHostedApiUrl(url: string): boolean {
  if (isRetiredNozyHostedApiUrl(url)) return true;
  if (isLocalApiUrl(url)) return false;
  return /^https:\/\//i.test(url.trim());
}

export function inferApiConnectionMode(url: string): ApiConnectionMode {
  return isHostedApiUrl(url) ? "hosted" : "self";
}

/** True when Zebrad RPC is loopback / LAN (Case A1). */
export function isLocalZebraUrl(url: string): boolean {
  const host = hostFromUrl(url);
  if (!host) return false;
  return isPrivateLanHost(host);
}

export function inferNodeConnectionMode(url: string): NodeConnectionMode {
  return isLocalZebraUrl(url) ? "local" : "public";
}
