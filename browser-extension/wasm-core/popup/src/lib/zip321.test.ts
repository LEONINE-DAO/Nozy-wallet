import { describe, expect, it } from "vitest";
import {
  amountZecToZats,
  buildZip321Uri,
  extractZip321Uri,
  parseZip321Uri,
  tryParsePaymentInput,
} from "./zip321";

const UA =
  "u1testaddressplaceholder00000000000000000000000000000000000000000000000000000000000";

describe("zip321", () => {
  it("round-trips amount and base64url memo", () => {
    const uri = buildZip321Uri(UA, 0.05, "invoice-1");
    expect(uri.startsWith("zcash:u1")).toBe(true);
    expect(uri).toContain("amount=0.05");
    expect(uri).not.toContain("memo=invoice-1");
    const parsed = parseZip321Uri(uri);
    expect(parsed.amountZec).toBe("0.05");
    expect(parsed.memo).toBe("invoice-1");
  });

  it("decodes the ZIP-321 spec memo example", () => {
    const uri = `zcash:${UA}?amount=1&memo=VGhpcyBpcyBhIHNpbXBsZSBtZW1vLg&message=Thank%20you%20for%20your%20purchase&label=Shop`;
    const p = parseZip321Uri(uri);
    expect(p.memo).toBe("This is a simple memo.");
    expect(p.message).toBe("Thank you for your purchase");
    expect(p.label).toBe("Shop");
  });

  it("accepts address= query form", () => {
    const p = parseZip321Uri(`zcash:?address=${UA}&amount=0.25`);
    expect(p.address).toBe(UA);
    expect(p.amountZec).toBe("0.25");
  });

  it("rejects transparent, sapling, //, req-, and ZGo page URLs", () => {
    expect(() => parseZip321Uri("zcash:tmEZhbWHTpdKMw5it8YDspUXSMGQyFwovpU?amount=1")).toThrow(
      /transparent/i,
    );
    expect(() =>
      parseZip321Uri(
        "zcash:ztestsapling10yy2ex5dcqkclhc7z7yrnjq2z6feyjad56ptwlfgmy77dmaqqrl9gyhprdx59qgmsnyfska2kez?amount=1&memo=VGhpcyBpcyBhIHNpbXBsZSBtZW1vLg",
      ),
    ).toThrow(/unified/i);
    expect(() => parseZip321Uri(`zcash://${UA}?amount=1`)).toThrow(/\/\//);
    expect(() => parseZip321Uri(`zcash:${UA}?amount=1&req-asset=abc`)).toThrow(/req-asset/);
    expect(() => tryParsePaymentInput("https://app.zgo.cash/pmtservice?owner=abc")).toThrow(
      /does not call ZGo/i,
    );
    expect(extractZip321Uri("https://app.zgo.cash/pmtservice?owner=abc")).toBeNull();
  });

  it("extracts a URI from wrapped clipboard text", () => {
    const inner = buildZip321Uri(UA, 1, "hi");
    const p = parseZip321Uri(`Pay here: ${inner} thanks`);
    expect(p.amountZec).toBe("1");
    expect(p.memo).toBe("hi");
  });

  it("converts ZEC amount to zats for the extension send field", () => {
    expect(amountZecToZats("0.05")).toBe("5000000");
  });

  it("leaves ordinary addresses unparsed", () => {
    expect(tryParsePaymentInput(UA)).toBeNull();
  });
});
