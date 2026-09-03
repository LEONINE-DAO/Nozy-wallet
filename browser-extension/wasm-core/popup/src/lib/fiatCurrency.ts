/** Fiat codes CoinGecko `simple/price` accepts for ZEC. */

export type FiatCurrency = {
  code: string;
  name: string;
  symbol: string;
};

export const FIAT_CURRENCIES: FiatCurrency[] = [
  { code: "USD", name: "US Dollar", symbol: "$" },
  { code: "EUR", name: "Euro", symbol: "€" },
  { code: "GBP", name: "British Pound", symbol: "£" },
  { code: "JPY", name: "Japanese Yen", symbol: "¥" },
  { code: "KRW", name: "South Korean Won", symbol: "₩" },
  { code: "CNY", name: "Chinese Yuan", symbol: "¥" },
  { code: "AUD", name: "Australian Dollar", symbol: "A$" },
  { code: "CAD", name: "Canadian Dollar", symbol: "C$" },
  { code: "CHF", name: "Swiss Franc", symbol: "Fr" },
  { code: "HKD", name: "Hong Kong Dollar", symbol: "HK$" },
  { code: "SGD", name: "Singapore Dollar", symbol: "S$" },
  { code: "INR", name: "Indian Rupee", symbol: "₹" },
  { code: "BRL", name: "Brazilian Real", symbol: "R$" },
  { code: "MXN", name: "Mexican Peso", symbol: "MX$" },
  { code: "NZD", name: "New Zealand Dollar", symbol: "NZ$" },
  { code: "ZAR", name: "South African Rand", symbol: "R" },
  { code: "SEK", name: "Swedish Krona", symbol: "kr" },
  { code: "NOK", name: "Norwegian Krone", symbol: "kr" },
  { code: "DKK", name: "Danish Krone", symbol: "kr" },
  { code: "PLN", name: "Polish Zloty", symbol: "zł" },
  { code: "CZK", name: "Czech Koruna", symbol: "Kč" },
  { code: "HUF", name: "Hungarian Forint", symbol: "Ft" },
  { code: "RON", name: "Romanian Leu", symbol: "lei" },
  { code: "TRY", name: "Turkish Lira", symbol: "₺" },
  { code: "AED", name: "UAE Dirham", symbol: "د.إ" },
  { code: "SAR", name: "Saudi Riyal", symbol: "﷼" },
  { code: "ILS", name: "Israeli Shekel", symbol: "₪" },
  { code: "THB", name: "Thai Baht", symbol: "฿" },
  { code: "IDR", name: "Indonesian Rupiah", symbol: "Rp" },
  { code: "PHP", name: "Philippine Peso", symbol: "₱" },
  { code: "MYR", name: "Malaysian Ringgit", symbol: "RM" },
  { code: "VND", name: "Vietnamese Dong", symbol: "₫" },
  { code: "TWD", name: "New Taiwan Dollar", symbol: "NT$" },
  { code: "ARS", name: "Argentine Peso", symbol: "AR$" },
  { code: "CLP", name: "Chilean Peso", symbol: "CL$" },
  { code: "COP", name: "Colombian Peso", symbol: "CO$" },
  { code: "PEN", name: "Peruvian Sol", symbol: "S/" },
  { code: "NGN", name: "Nigerian Naira", symbol: "₦" },
  { code: "EGP", name: "Egyptian Pound", symbol: "E£" },
  { code: "PKR", name: "Pakistani Rupee", symbol: "₨" },
  { code: "BDT", name: "Bangladeshi Taka", symbol: "৳" },
  { code: "UAH", name: "Ukrainian Hryvnia", symbol: "₴" },
  { code: "KZT", name: "Kazakhstani Tenge", symbol: "₸" }
];

export const FIAT_CODES = FIAT_CURRENCIES.map((c) => c.code);
const FIAT_SET = new Set(FIAT_CODES);

const REGION_CURRENCY: Record<string, string> = {
  US: "USD",
  PR: "USD",
  GU: "USD",
  VI: "USD",
  AS: "USD",
  MP: "USD",
  GB: "GBP",
  UK: "GBP",
  IE: "EUR",
  DE: "EUR",
  FR: "EUR",
  ES: "EUR",
  IT: "EUR",
  NL: "EUR",
  BE: "EUR",
  AT: "EUR",
  PT: "EUR",
  FI: "EUR",
  GR: "EUR",
  SK: "EUR",
  SI: "EUR",
  EE: "EUR",
  LV: "EUR",
  LT: "EUR",
  LU: "EUR",
  MT: "EUR",
  CY: "EUR",
  HR: "EUR",
  JP: "JPY",
  KR: "KRW",
  CN: "CNY",
  HK: "HKD",
  TW: "TWD",
  SG: "SGD",
  AU: "AUD",
  NZ: "NZD",
  CA: "CAD",
  CH: "CHF",
  LI: "CHF",
  IN: "INR",
  BR: "BRL",
  MX: "MXN",
  ZA: "ZAR",
  SE: "SEK",
  NO: "NOK",
  DK: "DKK",
  PL: "PLN",
  CZ: "CZK",
  HU: "HUF",
  RO: "RON",
  TR: "TRY",
  AE: "AED",
  SA: "SAR",
  IL: "ILS",
  TH: "THB",
  ID: "IDR",
  PH: "PHP",
  MY: "MYR",
  VN: "VND",
  AR: "ARS",
  CL: "CLP",
  CO: "COP",
  PE: "PEN",
  NG: "NGN",
  EG: "EGP",
  PK: "PKR",
  BD: "BDT",
  UA: "UAH",
  KZ: "KZT"
};

const TZ_CURRENCY: Array<[string, string]> = [
  ["America/New_York", "USD"],
  ["America/Chicago", "USD"],
  ["America/Denver", "USD"],
  ["America/Los_Angeles", "USD"],
  ["America/Phoenix", "USD"],
  ["Pacific/Honolulu", "USD"],
  ["America/Toronto", "CAD"],
  ["America/Vancouver", "CAD"],
  ["America/Sao_Paulo", "BRL"],
  ["America/Mexico_City", "MXN"],
  ["America/Bogota", "COP"],
  ["America/Santiago", "CLP"],
  ["America/Lima", "PEN"],
  ["America/Argentina/Buenos_Aires", "ARS"],
  ["Europe/London", "GBP"],
  ["Europe/Dublin", "EUR"],
  ["Europe/Paris", "EUR"],
  ["Europe/Berlin", "EUR"],
  ["Europe/Madrid", "EUR"],
  ["Europe/Rome", "EUR"],
  ["Europe/Amsterdam", "EUR"],
  ["Europe/Zurich", "CHF"],
  ["Europe/Stockholm", "SEK"],
  ["Europe/Oslo", "NOK"],
  ["Europe/Copenhagen", "DKK"],
  ["Europe/Warsaw", "PLN"],
  ["Europe/Prague", "CZK"],
  ["Europe/Budapest", "HUF"],
  ["Europe/Bucharest", "RON"],
  ["Europe/Istanbul", "TRY"],
  ["Europe/Kyiv", "UAH"],
  ["Asia/Tokyo", "JPY"],
  ["Asia/Seoul", "KRW"],
  ["Asia/Shanghai", "CNY"],
  ["Asia/Hong_Kong", "HKD"],
  ["Asia/Taipei", "TWD"],
  ["Asia/Singapore", "SGD"],
  ["Asia/Kolkata", "INR"],
  ["Asia/Calcutta", "INR"],
  ["Asia/Dubai", "AED"],
  ["Asia/Riyadh", "SAR"],
  ["Asia/Jerusalem", "ILS"],
  ["Asia/Bangkok", "THB"],
  ["Asia/Jakarta", "IDR"],
  ["Asia/Manila", "PHP"],
  ["Asia/Kuala_Lumpur", "MYR"],
  ["Asia/Ho_Chi_Minh", "VND"],
  ["Asia/Karachi", "PKR"],
  ["Asia/Dhaka", "BDT"],
  ["Asia/Almaty", "KZT"],
  ["Australia/Sydney", "AUD"],
  ["Australia/Melbourne", "AUD"],
  ["Pacific/Auckland", "NZD"],
  ["Africa/Johannesburg", "ZAR"],
  ["Africa/Lagos", "NGN"],
  ["Africa/Cairo", "EGP"]
];

export function isFiatCode(code: string): boolean {
  return FIAT_SET.has(code.toUpperCase());
}

export function fiatByCode(code: string): FiatCurrency | undefined {
  const c = code.toUpperCase();
  return FIAT_CURRENCIES.find((x) => x.code === c);
}

/** Locale region (e.g. en-US → US) then timezone, else USD. */
export function detectDefaultCurrency(opts?: {
  locale?: string;
  timeZone?: string;
}): string {
  const locale = opts?.locale ?? (typeof navigator !== "undefined" ? navigator.language : "en-US");
  const region = locale.split(/[-_]/)[1]?.toUpperCase();
  if (region && REGION_CURRENCY[region] && isFiatCode(REGION_CURRENCY[region])) {
    return REGION_CURRENCY[region];
  }
  let tz = opts?.timeZone;
  if (!tz) {
    try {
      tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    } catch {
      tz = "";
    }
  }
  if (tz) {
    const exact = TZ_CURRENCY.find(([name]) => name === tz);
    if (exact) return exact[1];
  }
  return "USD";
}
