import { describe, expect, it } from "vitest";
import { detectDefaultCurrency } from "./fiatCurrency";
import { fiatForZec, formatFiat, formatUsd } from "./zecPrice";

describe("zecPrice", () => {
  it("formats USD", () => {
    expect(formatUsd(42.1)).toBe("$42.10");
  });

  it("formats EUR", () => {
    const s = formatFiat(40, "EUR");
    expect(s).toMatch(/40/);
    expect(s).toMatch(/€|EUR/);
  });

  it("converts a ZEC amount to fiat", () => {
    expect(fiatForZec(2, 40, "USD")).toBe("$80.00");
    expect(fiatForZec(1, null, "USD")).toBe(null);
  });
});

describe("detectDefaultCurrency", () => {
  it("uses locale region", () => {
    expect(detectDefaultCurrency({ locale: "en-GB", timeZone: "UTC" })).toBe("GBP");
    expect(detectDefaultCurrency({ locale: "ja-JP", timeZone: "UTC" })).toBe("JPY");
    expect(detectDefaultCurrency({ locale: "de-DE", timeZone: "UTC" })).toBe("EUR");
    expect(detectDefaultCurrency({ locale: "pt-BR", timeZone: "UTC" })).toBe("BRL");
  });

  it("falls back to timezone when locale has no region", () => {
    expect(detectDefaultCurrency({ locale: "en", timeZone: "Asia/Tokyo" })).toBe("JPY");
    expect(detectDefaultCurrency({ locale: "en", timeZone: "Europe/London" })).toBe("GBP");
  });

  it("defaults to USD", () => {
    expect(detectDefaultCurrency({ locale: "en", timeZone: "Etc/UTC" })).toBe("USD");
  });
});
