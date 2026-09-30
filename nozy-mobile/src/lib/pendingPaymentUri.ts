/** Deep-link / QR payload held until Send is on screen. ZIP-321 only — no ZGo HTTP. */

let pending: string | null = null;

export function setPendingPaymentUri(uri: string): void {
  pending = uri.trim() || null;
}

export function takePendingPaymentUri(): string | null {
  const value = pending;
  pending = null;
  return value;
}
