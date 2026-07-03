/**
 * ZEC price from CoinGecko (no API key). Cached 5 minutes for all supported fiats.
 */

import {
  type FiatCurrency,
  allCoingeckoKeys,
  coingeckoKeyFor,
  fiatLocale,
} from "../lib/fiatCurrencies";

export type { FiatCurrency } from "../lib/fiatCurrencies";

const CACHE_MS = 5 * 60 * 1000;
let cache: { timestamp: number; rates: Partial<Record<string, number>> } | null = null;

async function fetchRates(): Promise<Partial<Record<string, number>> | null> {
  const now = Date.now();
  if (cache && now - cache.timestamp < CACHE_MS) {
    return cache.rates;
  }
  try {
    const res = await fetch(
      `https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=${allCoingeckoKeys()}`
    );
    if (!res.ok) return cache?.rates ?? null;
    const data = (await res.json()) as { zcash?: Record<string, number> };
    const zcash = data?.zcash;
    if (!zcash || typeof zcash !== "object") return cache?.rates ?? null;
    cache = { timestamp: now, rates: zcash };
    return zcash;
  } catch {
    return cache?.rates ?? null;
  }
}

export async function getZecPriceInFiat(currency: FiatCurrency): Promise<number | null> {
  const rates = await fetchRates();
  if (!rates) return null;
  const key = coingeckoKeyFor(currency);
  const rate = rates[key];
  return typeof rate === "number" && rate > 0 ? rate : null;
}

export function formatFiatAmount(amount: number, currency: FiatCurrency): string {
  const fractionDigits = ["JPY", "KRW", "VND", "IDR", "CLP"].includes(currency) ? 0 : 2;
  try {
    return new Intl.NumberFormat(fiatLocale(currency), {
      style: "currency",
      currency,
      minimumFractionDigits: fractionDigits,
      maximumFractionDigits: fractionDigits,
    }).format(amount);
  } catch {
    return `${currency} ${amount.toFixed(fractionDigits)}`;
  }
}
