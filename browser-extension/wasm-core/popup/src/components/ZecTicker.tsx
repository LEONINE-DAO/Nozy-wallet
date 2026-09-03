import zecMark from "../assets/zec.svg";
import { fiatForZec, formatFiat, useZecFiatPrice } from "../lib/zecPrice";

/** Compact Home ticker — Zcash mark + unit price in the selected fiat. */
export function ZecTicker({ zecAmount }: { zecAmount: number | null }) {
  const { currency, rate } = useZecFiatPrice();
  const unit = rate != null ? `1 ZEC ≈ ${formatFiat(rate, currency)}` : "Price unavailable";
  const fiat = zecAmount != null ? fiatForZec(zecAmount, rate, currency) : null;

  return (
    <div className="nw-zec-ticker">
      <img className="nw-zec-ticker__icon" src={zecMark} alt="Zcash" />
      <div className="min-w-0 flex-1">
        <p className="text-[13px] font-semibold leading-tight">Zcash</p>
        <p className="nw-hint mt-0.5 truncate">{unit}</p>
      </div>
      {fiat ? (
        <p className="shrink-0 text-[13px] font-semibold tabular-nums" style={{ color: "var(--nw-platinum)" }}>
          {fiat}
        </p>
      ) : null}
    </div>
  );
}
