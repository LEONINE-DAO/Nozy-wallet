export type ZcashPriceQuote = {
  usd: number;
  change24h: number | null;
};

const COINGECKO_URL =
  "https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd&include_24hr_change=true";

export async function fetchZcashPrice(): Promise<ZcashPriceQuote> {
  const response = await fetch(COINGECKO_URL);
  if (!response.ok) {
    throw new Error(`price fetch failed (${response.status})`);
  }

  const data = (await response.json()) as {
    zcash?: { usd?: number; usd_24h_change?: number };
  };

  const usd = data.zcash?.usd;
  if (typeof usd !== "number" || !Number.isFinite(usd)) {
    throw new Error("invalid price payload");
  }

  const rawChange = data.zcash?.usd_24h_change;
  const change24h =
    typeof rawChange === "number" && Number.isFinite(rawChange) ? rawChange : null;

  return { usd, change24h };
}

export function formatUsdPrice(value: number): string {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: value >= 100 ? 2 : 4,
  }).format(value);
}

export function formatPercentChange(value: number): string {
  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
}
