/** Consumer NymVPN (OS app). Not in-extension mixnet. */

export const ZCASH_NYM_FREE_URL = "https://zcash.nym.com";
export const NYM_VPN_APP_URL = "https://nym.com/vpn";

/** Claim/install pages must open without a tunnel so the user can get NymVPN. */
export function isNymVpnOnrampUrl(raw: string): boolean {
  try {
    const u = new URL(raw);
    if (u.protocol !== "https:" && u.protocol !== "http:") return false;
    const h = u.hostname.replace(/^www\./i, "").toLowerCase();
    return h === "zcash.nym.com" || h === "nym.com" || h.endsWith(".nym.com");
  } catch {
    return false;
  }
}
