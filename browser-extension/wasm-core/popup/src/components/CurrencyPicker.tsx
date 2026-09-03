import { useEffect, useMemo, useRef, useState } from "react";
import { FIAT_CURRENCIES, fiatByCode } from "../lib/fiatCurrency";
import { Icon } from "./ui";

export function CurrencyPicker({
  currency,
  suggested,
  onChange
}: {
  currency: string;
  suggested: string;
  onChange: (code: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const rootRef = useRef<HTMLDivElement>(null);
  const current = fiatByCode(currency);

  useEffect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      if (!rootRef.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = !q
      ? FIAT_CURRENCIES
      : FIAT_CURRENCIES.filter(
          (c) =>
            c.code.toLowerCase().includes(q) ||
            c.name.toLowerCase().includes(q) ||
            c.symbol.toLowerCase().includes(q)
        );
    const sug = list.find((c) => c.code === suggested);
    const rest = list.filter((c) => c.code !== suggested);
    return sug ? [sug, ...rest] : list;
  }, [query, suggested]);

  return (
    <div className="nw-fx" ref={rootRef}>
      <button
        type="button"
        className="nw-keplr__usd"
        aria-haspopup="listbox"
        aria-expanded={open}
        title="Display currency"
        onClick={() => setOpen((v) => !v)}
      >
        {current?.code ?? "USD"}
        <Icon name="chevron" size={12} />
      </button>
      {open ? (
        <div className="nw-fx__menu" role="listbox" aria-label="Currencies">
          <input
            className="nw-fx__search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search currency"
            autoFocus
          />
          <div className="nw-fx__list">
            {filtered.map((c) => (
              <button
                key={c.code}
                type="button"
                role="option"
                aria-selected={c.code === currency}
                className={`nw-fx__opt${c.code === currency ? " nw-fx__opt--on" : ""}`}
                onClick={() => {
                  onChange(c.code);
                  setOpen(false);
                  setQuery("");
                }}
              >
                <span className="nw-fx__code">{c.code}</span>
                <span className="nw-fx__name">
                  {c.name}
                  {c.code === suggested ? " · local" : ""}
                </span>
                <span className="nw-fx__sym">{c.symbol}</span>
              </button>
            ))}
            {filtered.length === 0 ? <p className="nw-fx__empty">No match</p> : null}
          </div>
        </div>
      ) : null}
    </div>
  );
}
