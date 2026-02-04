/**
 * ZEC price from CoinGecko (no API key). Cached 5 minutes.
 */

const CACHE_MS = 5 * 60 * 1000;
let cache: { timestamp: number; usd: number; eur: number } | null = null;

export type FiatCurrency = "USD" | "EUR";

const CURRENCY_KEY: Record<FiatCurrency, keyof { usd: number; eur: number }> = {
  USD: "usd",
  EUR: "eur",
};

export async function getZecPriceInFiat(currency: FiatCurrency): Promise<number | null> {
  const key = CURRENCY_KEY[currency];
  const now = Date.now();
  if (cache && now - cache.timestamp < CACHE_MS && typeof cache[key] === "number") {
    return cache[key];
  }
  try {
    const res = await fetch(
      "https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd,eur"
    );
    if (!res.ok) return null;
    const data = (await res.json()) as { zcash?: { usd?: number; eur?: number } };
    const zcash = data?.zcash;
    if (!zcash || typeof zcash.usd !== "number" || typeof zcash.eur !== "number") return null;
    cache = { timestamp: now, usd: zcash.usd, eur: zcash.eur };
    return cache[key];
  } catch {
    return null;
  }
}

export function formatFiatAmount(amount: number, currency: FiatCurrency): string {
  const symbol = currency === "USD" ? "$" : "€";
  return `${symbol}${amount.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}
