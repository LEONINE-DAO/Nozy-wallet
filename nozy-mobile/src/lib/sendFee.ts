/** Matches the phone send in `send_payment_via_lightwalletd` (ZIP-317 × 4, change output). */
const MARGINAL_FEE_ZATS = 5_000;
const GRACE_ACTIONS = 2;
const PRIORITY_MULTIPLIER = 4;
const ZATS_PER_ZEC = 100_000_000;

function utf8Length(text: string): number {
  let bytes = 0;
  for (const char of text) {
    const code = char.codePointAt(0) ?? 0;
    if (code < 0x80) bytes += 1;
    else if (code < 0x800) bytes += 2;
    else if (code < 0x10000) bytes += 3;
    else bytes += 4;
  }
  return bytes;
}

/** Network fee in zatoshis for one shielded spend with a change output. */
export function phoneSendFeeZats(memo: string): number {
  const memoLen = utf8Length(memo.trim());
  const memoChunks = memoLen === 0 ? 0 : Math.ceil(memoLen / 512);
  const logical = GRACE_ACTIONS + Math.max(0, memoChunks - 2);
  const billable = Math.max(logical, GRACE_ACTIONS);
  return MARGINAL_FEE_ZATS * billable * PRIORITY_MULTIPLIER;
}

/** A lone comma is a decimal separator (`0,169` is 0.169 ZEC). */
export function normalizeZecAmountInput(raw: string): string {
  const compact = raw.replace(/\s/g, "");
  if (compact.includes(",") && !compact.includes(".")) {
    return compact.replace(",", ".");
  }
  return compact;
}

export function parseZecAmount(raw: string): number {
  return parseFloat(normalizeZecAmountInput(raw));
}

export function zecToZats(zec: number): number {
  if (!Number.isFinite(zec) || zec <= 0) return 0;
  return Math.round(zec * ZATS_PER_ZEC);
}

export function formatZats(zats: number): string {
  return (Math.max(0, zats) / ZATS_PER_ZEC).toFixed(8);
}

/** Largest amount that still leaves `feeZats` inside `sourceZats`. */
export function maxSendableZats(sourceZats: number, feeZats: number): number {
  return Math.max(0, sourceZats - feeZats);
}

export function feeExceedsSourceMessage(feeZats: number, maxSendZats: number, singleNote: boolean): string {
  const fee = formatZats(feeZats);
  const max = formatZats(maxSendZats);
  const limit = singleNote
    ? `This phone spends one note at a time. After the ${fee} ZEC network fee, the most you can send is ${max} ZEC.`
    : `The network fee is ${fee} ZEC. The most you can send is ${max} ZEC.`;
  return limit;
}
