/**
 * Live ZEC/fiat from CoinGecko (same source as Desktop).
 * Cached in chrome.storage so a closed popup does not refetch every open.
 */

import { useCallback, useEffect, useState } from "react";
import {
  detectDefaultCurrency,
  FIAT_CODES,
  isFiatCode
} from "./fiatCurrency";

const PRICE_CACHE_KEY = "nozy_zec_fiat_price_v1";
const CURRENCY_PREF_KEY = "nozy_fiat_currency_v1";
const CACHE_MS = 5 * 60 * 1000;
const VS = FIAT_CODES.map((c) => c.toLowerCase()).join(",");
const COINGECKO_URL = `https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=${VS}`;

export type ZecFiatCache = {
  rates: Record<string, number>;
  ts: number;
};

let memRates: ZecFiatCache | null = null;
let memCurrency: string | null = null;
let inflight: Promise<ZecFiatCache | null> | null = null;

function isUsableRates(cache: ZecFiatCache | null | undefined): cache is ZecFiatCache {
  return Boolean(cache && cache.rates && typeof cache.rates.usd === "number" && cache.rates.usd > 0);
}

function isFresh(cache: ZecFiatCache): boolean {
  return Date.now() - cache.ts < CACHE_MS;
}

async function storageGet<T>(key: string): Promise<T | undefined> {
  try {
    if (typeof chrome !== "undefined" && chrome.storage?.local) {
      const bag = await chrome.storage.local.get(key);
      return bag[key] as T | undefined;
    }
  } catch {
    /* ignore */
  }
  return undefined;
}

async function storageSet(key: string, value: unknown): Promise<void> {
  try {
    if (typeof chrome !== "undefined" && chrome.storage?.local) {
      await chrome.storage.local.set({ [key]: value });
    }
  } catch {
    /* ignore */
  }
}

async function readStoredRates(): Promise<ZecFiatCache | null> {
  if (memRates && isUsableRates(memRates)) return memRates;
  const cached = await storageGet<ZecFiatCache>(PRICE_CACHE_KEY);
  if (isUsableRates(cached)) {
    memRates = cached;
    return cached;
  }
  return memRates;
}

async function writeStoredRates(cache: ZecFiatCache): Promise<void> {
  memRates = cache;
  await storageSet(PRICE_CACHE_KEY, cache);
}

async function fetchCoinGeckoRates(): Promise<Record<string, number> | null> {
  try {
    const res = await fetch(COINGECKO_URL);
    if (!res.ok) return null;
    const data = (await res.json()) as { zcash?: Record<string, number> };
    const raw = data.zcash;
    if (!raw || typeof raw !== "object") return null;
    const rates: Record<string, number> = {};
    for (const [k, v] of Object.entries(raw)) {
      if (typeof v === "number" && v > 0 && Number.isFinite(v)) {
        rates[k.toLowerCase()] = v;
      }
    }
    return rates.usd ? rates : null;
  } catch {
    return null;
  }
}

async function loadRates(): Promise<ZecFiatCache | null> {
  if (inflight) return inflight;
  inflight = (async () => {
    const cached = await readStoredRates();
    if (cached && isFresh(cached)) return cached;
    const rates = await fetchCoinGeckoRates();
    if (rates) {
      const next = { rates, ts: Date.now() };
      await writeStoredRates(next);
      return next;
    }
    return cached;
  })().finally(() => {
    inflight = null;
  });
  return inflight;
}

export async function loadFiatPreference(): Promise<string> {
  if (memCurrency && isFiatCode(memCurrency)) return memCurrency;
  const stored = await storageGet<string>(CURRENCY_PREF_KEY);
  if (typeof stored === "string" && isFiatCode(stored)) {
    memCurrency = stored.toUpperCase();
    return memCurrency;
  }
  const guessed = detectDefaultCurrency();
  memCurrency = guessed;
  return guessed;
}

export async function saveFiatPreference(code: string): Promise<void> {
  const next = code.toUpperCase();
  if (!isFiatCode(next)) return;
  memCurrency = next;
  await storageSet(CURRENCY_PREF_KEY, next);
}

export function formatFiat(amount: number, currency = "USD"): string {
  const code = isFiatCode(currency) ? currency.toUpperCase() : "USD";
  try {
    const zeroDecimal = code === "JPY" || code === "KRW" || code === "VND" || code === "CLP" || code === "HUF";
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency: code,
      minimumFractionDigits: zeroDecimal ? 0 : 2,
      maximumFractionDigits: zeroDecimal ? 0 : 2
    }).format(amount);
  } catch {
    return `${code} ${amount.toFixed(2)}`;
  }
}

/** @deprecated Prefer formatFiat — kept for USD-only call sites and tests. */
export function formatUsd(amount: number): string {
  return formatFiat(amount, "USD");
}

export function fiatForZec(
  zec: number,
  rate: number | null,
  currency = "USD"
): string | null {
  if (rate == null || rate <= 0 || !Number.isFinite(zec)) return null;
  return formatFiat(zec * rate, currency);
}

export function rateForCurrency(rates: Record<string, number> | null | undefined, currency: string): number | null {
  if (!rates) return null;
  const v = rates[currency.toLowerCase()];
  return typeof v === "number" && v > 0 ? v : null;
}

/** Cached rates + selected fiat (locale/timezone default until the user picks). */
export function useZecFiatPrice(): {
  currency: string;
  suggested: string;
  rate: number | null;
  setCurrency: (code: string) => void;
} {
  const suggested = detectDefaultCurrency();
  const [currency, setCurrencyState] = useState(suggested);
  const [rate, setRate] = useState<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const pref = await loadFiatPreference();
      if (!cancelled) setCurrencyState(pref);
      const cached = await readStoredRates();
      if (!cancelled && cached) setRate(rateForCurrency(cached.rates, pref));
      const next = await loadRates();
      if (!cancelled && next) {
        setRate(rateForCurrency(next.rates, memCurrency ?? pref));
      }
    })();

    const onStorage = (
      changes: { [key: string]: chrome.storage.StorageChange },
      area: string
    ) => {
      if (area !== "local" || !changes[CURRENCY_PREF_KEY]) return;
      const next = changes[CURRENCY_PREF_KEY].newValue;
      if (typeof next === "string" && isFiatCode(next)) {
        const code = next.toUpperCase();
        memCurrency = code;
        setCurrencyState(code);
        setRate(rateForCurrency(memRates?.rates, code));
      }
    };
    try {
      chrome.storage?.onChanged?.addListener(onStorage);
    } catch {
      /* ignore */
    }
    return () => {
      cancelled = true;
      try {
        chrome.storage?.onChanged?.removeListener(onStorage);
      } catch {
        /* ignore */
      }
    };
  }, []);

  const setCurrency = useCallback((code: string) => {
    if (!isFiatCode(code)) return;
    const next = code.toUpperCase();
    setCurrencyState(next);
    void saveFiatPreference(next);
    const rates = memRates?.rates;
    setRate(rateForCurrency(rates, next));
    void loadRates().then((cache) => {
      if (cache) setRate(rateForCurrency(cache.rates, next));
    });
  }, []);

  return { currency, suggested, rate, setCurrency };
}

/** USD-only helper used by older call sites. */
export function useZecUsdPrice(): number | null {
  const { currency, rate } = useZecFiatPrice();
  if (currency !== "USD") return rate;
  return rate;
}
