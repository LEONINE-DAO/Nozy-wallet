import type { WalletScanProgressResult } from "./extensionApi";

/**
 * Fractional percent computed from raw block counts. The background rounds `percent` to two
 * decimals, which collapses to 0.00 on million-block scans, so prefer the counts when present.
 */
export function scanPercentDisplay(scan: WalletScanProgressResult | null | undefined): number {
  if (!scan) return 0;
  const done = scan.scannedBlocks;
  const total = scan.totalBlocks;
  if (typeof done === "number" && typeof total === "number" && total > 0) {
    return Math.min(100, Math.max(0, (done / total) * 100));
  }
  const pct = typeof scan.percent === "number" ? scan.percent : (scan.percentInt ?? 0);
  return Math.min(100, Math.max(0, pct));
}

/** True only while a scan is running and has not reached 100%. Used to show/hide the chain-sync popup. */
export function isScanInProgress(scan: WalletScanProgressResult | null | undefined): boolean {
  if (!scan || scan.status !== "scanning") return false;
  const done = scan.scannedBlocks;
  const total = scan.totalBlocks;
  if (typeof done === "number" && typeof total === "number" && total > 0 && done >= total) {
    return false;
  }
  return scanPercentDisplay(scan) < 100;
}

/** Same floor as desktop `MAINNET_DEFAULT_SCAN_START` for the original wallet only. */
export const MAINNET_RESTORE_SCAN_FLOOR = 3_050_000;

/** Empty wallet that only covered a recent slice. */
export function scanLooksLikeEmptyNearTipWindow(
  scan: WalletScanProgressResult | null | undefined,
  restoreFloor = MAINNET_RESTORE_SCAN_FLOOR
): boolean {
  if (!scan) return false;
  if ((scan.discoveredNotes ?? 0) > 0) return false;
  const start = scan.startHeight;
  const end = scan.endHeight;
  if (typeof start !== "number" || typeof end !== "number") return false;
  return start > restoreFloor + 1000 && end - start < 80_000;
}

/** Scan is running before this wallet's own birthday — jump forward to that height. */
export function shouldJumpToOwnBirthday(
  scan: WalletScanProgressResult | null | undefined,
  birthdayHeight: number | null | undefined
): boolean {
  if (typeof birthdayHeight !== "number" || birthdayHeight < 0) return false;
  const start = scan?.startHeight;
  if (typeof start !== "number") return false;
  return start < birthdayHeight - 64;
}

/** Adaptive precision so sub-1% progress on a long scan is still visible instead of reading 0%. */
export function scanPercentLabel(scan: WalletScanProgressResult | null | undefined): string {
  const pct = scanPercentDisplay(scan);
  if (pct <= 0) return "0";
  if (pct >= 10) return String(Math.floor(pct));
  if (pct >= 1) return pct.toFixed(1);
  if (pct >= 0.01) return pct.toFixed(2);
  return pct.toFixed(4);
}

export function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "—";
  if (seconds < 90) return `${Math.max(1, Math.round(seconds))}s`;
  const minutes = seconds / 60;
  if (minutes < 90) return `${Math.round(minutes)} min`;
  const hours = minutes / 60;
  if (hours < 36) return `${hours.toFixed(1)} h`;
  return `${Math.round(hours / 24)} d`;
}

/** Throughput + ETA so a long scan is visibly making progress even while percent rounds to 0. */
export function scanRateLabel(scan: WalletScanProgressResult | null | undefined): string | null {
  if (!scan || scan.status !== "scanning") return null;
  const done = scan.scannedBlocks ?? 0;
  const total = scan.totalBlocks ?? 0;
  const elapsedMs = scan.elapsed ?? 0;
  if (done <= 0 || elapsedMs <= 0) return null;
  const perSecond = done / (elapsedMs / 1000);
  if (!Number.isFinite(perSecond) || perSecond <= 0) return null;
  const remaining = Math.max(0, total - done);
  return `${Math.round(perSecond)} blk/s · ~${formatDuration(remaining / perSecond)} left`;
}
