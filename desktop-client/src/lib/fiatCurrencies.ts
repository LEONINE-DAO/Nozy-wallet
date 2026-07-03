/** Supported fiat display currencies (CoinGecko `vs_currencies` keys). */
export const FIAT_CURRENCIES = [
  { code: "USD", name: "US Dollar", coingecko: "usd", locale: "en-US" },
  { code: "EUR", name: "Euro", coingecko: "eur", locale: "de-DE" },
  { code: "GBP", name: "British Pound", coingecko: "gbp", locale: "en-GB" },
  { code: "JPY", name: "Japanese Yen", coingecko: "jpy", locale: "ja-JP" },
  { code: "CNY", name: "Chinese Yuan", coingecko: "cny", locale: "zh-CN" },
  { code: "CHF", name: "Swiss Franc", coingecko: "chf", locale: "de-CH" },
  { code: "CAD", name: "Canadian Dollar", coingecko: "cad", locale: "en-CA" },
  { code: "AUD", name: "Australian Dollar", coingecko: "aud", locale: "en-AU" },
  { code: "NZD", name: "New Zealand Dollar", coingecko: "nzd", locale: "en-NZ" },
  { code: "SGD", name: "Singapore Dollar", coingecko: "sgd", locale: "en-SG" },
  { code: "HKD", name: "Hong Kong Dollar", coingecko: "hkd", locale: "en-HK" },
  { code: "INR", name: "Indian Rupee", coingecko: "inr", locale: "en-IN" },
  { code: "KRW", name: "South Korean Won", coingecko: "krw", locale: "ko-KR" },
  { code: "BRL", name: "Brazilian Real", coingecko: "brl", locale: "pt-BR" },
  { code: "MXN", name: "Mexican Peso", coingecko: "mxn", locale: "es-MX" },
  { code: "ZAR", name: "South African Rand", coingecko: "zar", locale: "en-ZA" },
  { code: "NGN", name: "Nigerian Naira", coingecko: "ngn", locale: "en-NG" },
  { code: "GHS", name: "Ghanaian Cedi", coingecko: "ghs", locale: "en-GH" },
  { code: "KES", name: "Kenyan Shilling", coingecko: "kes", locale: "en-KE" },
  { code: "EGP", name: "Egyptian Pound", coingecko: "egp", locale: "ar-EG" },
  { code: "AED", name: "UAE Dirham", coingecko: "aed", locale: "ar-AE" },
  { code: "SAR", name: "Saudi Riyal", coingecko: "sar", locale: "ar-SA" },
  { code: "TRY", name: "Turkish Lira", coingecko: "try", locale: "tr-TR" },
  { code: "PLN", name: "Polish Zloty", coingecko: "pln", locale: "pl-PL" },
  { code: "SEK", name: "Swedish Krona", coingecko: "sek", locale: "sv-SE" },
  { code: "NOK", name: "Norwegian Krone", coingecko: "nok", locale: "nb-NO" },
  { code: "DKK", name: "Danish Krone", coingecko: "dkk", locale: "da-DK" },
  { code: "THB", name: "Thai Baht", coingecko: "thb", locale: "th-TH" },
  { code: "PHP", name: "Philippine Peso", coingecko: "php", locale: "en-PH" },
  { code: "IDR", name: "Indonesian Rupiah", coingecko: "idr", locale: "id-ID" },
  { code: "MYR", name: "Malaysian Ringgit", coingecko: "myr", locale: "ms-MY" },
  { code: "PKR", name: "Pakistani Rupee", coingecko: "pkr", locale: "en-PK" },
  { code: "BDT", name: "Bangladeshi Taka", coingecko: "bdt", locale: "bn-BD" },
  { code: "VND", name: "Vietnamese Dong", coingecko: "vnd", locale: "vi-VN" },
  { code: "ARS", name: "Argentine Peso", coingecko: "ars", locale: "es-AR" },
  { code: "COP", name: "Colombian Peso", coingecko: "cop", locale: "es-CO" },
  { code: "CLP", name: "Chilean Peso", coingecko: "clp", locale: "es-CL" },
  { code: "ILS", name: "Israeli Shekel", coingecko: "ils", locale: "he-IL" },
  { code: "RUB", name: "Russian Ruble", coingecko: "rub", locale: "ru-RU" },
] as const;

export type FiatCurrency = (typeof FIAT_CURRENCIES)[number]["code"];

export const DEFAULT_FIAT_CURRENCY: FiatCurrency = "USD";

const COINGECKO_BY_CODE = Object.fromEntries(
  FIAT_CURRENCIES.map((c) => [c.code, c.coingecko])
) as Record<FiatCurrency, string>;

const LOCALE_BY_CODE = Object.fromEntries(
  FIAT_CURRENCIES.map((c) => [c.code, c.locale])
) as Record<FiatCurrency, string>;

export function isFiatCurrency(value: unknown): value is FiatCurrency {
  return typeof value === "string" && value in COINGECKO_BY_CODE;
}

export function coingeckoKeyFor(currency: FiatCurrency): string {
  return COINGECKO_BY_CODE[currency];
}

export function allCoingeckoKeys(): string {
  return FIAT_CURRENCIES.map((c) => c.coingecko).join(",");
}

export function fiatLocale(currency: FiatCurrency): string {
  return LOCALE_BY_CODE[currency];
}
