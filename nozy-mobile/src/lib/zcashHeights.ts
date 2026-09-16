/** Mainnet NU5 / Orchard activation (ZIP 224). Same number as the extension scan shortcut. */
export const NU5_ORCHARD_MAINNET = 1_687_104;

/** Parse a restore-height field. Empty / invalid / genesis → NU5 (never height 1). */
export function parseRestoreHeight(raw?: string | number | null): number {
  if (raw == null || raw === "") return NU5_ORCHARD_MAINNET;
  const n =
    typeof raw === "number"
      ? raw
      : parseInt(String(raw).replace(/,/g, "").trim(), 10);
  if (!Number.isFinite(n) || n < NU5_ORCHARD_MAINNET) {
    return NU5_ORCHARD_MAINNET;
  }
  return Math.floor(n);
}
