export function isLikelyUnifiedOrchardAddress(value: string): boolean {
  return /^u1[0-9a-z]{20,}$/i.test(value);
}

export function memoByteLength(memo: string): number {
  return new TextEncoder().encode(memo).length;
}

export function validateMemoLimit(memo: string, maxBytes = 512): boolean {
  return memoByteLength(memo) <= maxBytes;
}

