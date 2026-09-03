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
