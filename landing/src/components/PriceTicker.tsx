import { useEffect, useState } from "react";
import {
  fetchZcashPrice,
  formatPercentChange,
  formatUsdPrice,
  type ZcashPriceQuote,
} from "../lib/zcashPrice";

/** Fixed strip height — keep in sync with Header `top-*` offset. */
export const TICKER_HEIGHT_PX = 32;

const REFRESH_MS = 60_000;

function TickerSegment({ quote }: { quote: ZcashPriceQuote | null }) {
  const priceLabel =
    quote != null ? formatUsdPrice(quote.usd) : "ZEC price loading…";

  const change =
    quote?.change24h != null ? (
      <span
        className={
          quote.change24h >= 0 ? "text-emerald-300" : "text-rose-300"
        }
      >
        {quote.change24h >= 0 ? "▲" : "▼"} {formatPercentChange(quote.change24h)}{" "}
        <span className="text-zinc-400">24h</span>
      </span>
    ) : (
      <span className="text-zinc-400">24h —</span>
    );

  return (
    <span className="inline-flex items-center gap-3 px-8 shrink-0">
      <span className="font-semibold text-yellow-300">ZEC</span>
      <span className="tabular-nums">{priceLabel}</span>
      {change}
      <span className="text-zinc-500" aria-hidden="true">
        ·
      </span>
      <span className="text-zinc-400">Zcash mainnet</span>
      <span className="text-zinc-500" aria-hidden="true">
        ·
      </span>
    </span>
  );
}

const PriceTicker = () => {
  const [quote, setQuote] = useState<ZcashPriceQuote | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      try {
        const next = await fetchZcashPrice();
        if (!cancelled) {
          setQuote(next);
        }
      } catch {
        /* keep last quote or loading state */
      }
    };

    void load();
    const timer = window.setInterval(() => void load(), REFRESH_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  const liveSummary =
    quote != null
      ? `Zcash ${formatUsdPrice(quote.usd)}${
          quote.change24h != null
            ? `, ${formatPercentChange(quote.change24h)} in 24 hours`
            : ""
        }`
      : "Zcash price loading";

  const segments = Array.from({ length: 8 }, (_, index) => (
    <TickerSegment key={index} quote={quote} />
  ));

  return (
    <div
      className="fixed top-0 left-0 right-0 z-[60] h-8 overflow-hidden border-b border-zinc-800/80 bg-zinc-950 text-[11px] sm:text-xs text-zinc-200"
      role="region"
      aria-label="Zcash price ticker"
    >
      <p className="sr-only" aria-live="polite">
        {liveSummary}
      </p>

      <div className="ticker-mask h-full flex items-center">
        <div className="ticker-track flex items-center">{segments}</div>
      </div>
    </div>
  );
};

export default PriceTicker;
