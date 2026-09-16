/**
 * ZEC/USD from CoinGecko (no API key). Cached 5 minutes — same source as desktop Home.
 */

const CACHE_MS = 5 * 60 * 1000;
let cache: { timestamp: number; usd: number } | null = null;

export async function getZecUsdPrice(): Promise<number | null> {
  const now = Date.now();
  if (cache && now - cache.timestamp < CACHE_MS) {
    return cache.usd;
  }
  try {
    const res = await fetch(
      "https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd",
    );
    if (!res.ok) return cache?.usd ?? null;
    const data = (await res.json()) as { zcash?: { usd?: number } };
    const usd = data?.zcash?.usd;
    if (typeof usd !== "number" || usd <= 0) return cache?.usd ?? null;
    cache = { timestamp: now, usd };
    return usd;
  } catch {
    return cache?.usd ?? null;
  }
}

export function formatUsd(amount: number): string {
  try {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(amount);
  } catch {
    return `$${amount.toFixed(2)}`;
  }
}
