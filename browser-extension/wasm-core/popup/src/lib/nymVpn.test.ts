import { describe, expect, it } from "vitest";
import { isNymVpnOnrampUrl, NYM_VPN_APP_URL, ZCASH_NYM_FREE_URL } from "./nymVpn";

describe("isNymVpnOnrampUrl", () => {
  it("allows claim and install hosts", () => {
    expect(isNymVpnOnrampUrl(ZCASH_NYM_FREE_URL)).toBe(true);
    expect(isNymVpnOnrampUrl(NYM_VPN_APP_URL)).toBe(true);
    expect(isNymVpnOnrampUrl("https://www.nym.com/vpn")).toBe(true);
  });

  it("does not treat general sites as onramp", () => {
    expect(isNymVpnOnrampUrl("https://z.cash")).toBe(false);
    expect(isNymVpnOnrampUrl("https://forum.zcashcommunity.com")).toBe(false);
    expect(isNymVpnOnrampUrl("javascript:alert(1)")).toBe(false);
  });
});
