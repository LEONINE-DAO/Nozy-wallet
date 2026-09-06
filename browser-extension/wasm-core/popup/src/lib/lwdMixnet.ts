/** Operator opt-in: Joaco lwd-mixnet-proxy dialling half on loopback.
 *  Not the Nym SDK in Chrome. Not a product default. */
export const MIXNET_LWD_GRPC_URL = "http://127.0.0.1:9068";
export const MIXNET_LWD_METRICS_URL = "http://127.0.0.1:9070";
export const DEFAULT_LOCAL_LWD_URL = "http://127.0.0.1:9067";

export function isMixnetLwdUrl(url: string | null | undefined): boolean {
  const u = String(url ?? "")
    .trim()
    .replace(/\/$/, "");
  return u === MIXNET_LWD_GRPC_URL || u === "http://localhost:9068";
}
