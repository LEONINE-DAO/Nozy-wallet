export type ZcashNetwork = "mainnet" | "testnet";

const MAINNET_EXPLORER_BASE = "https://mainnet.zcashexplorer.app";

/** Public block explorer transaction URL on Zcash Explorer. */
export function transactionExplorerUrl(
  txid: string,
  network: ZcashNetwork = "mainnet"
): string | null {
  const trimmed = txid.trim().replace(/^0x/i, "");
  if (!/^[0-9a-fA-F]{64}$/.test(trimmed)) {
    return null;
  }

  if (network === "testnet") {
    return `https://blockchair.com/zcash-testnet/transaction/${trimmed}`;
  }

  return `${MAINNET_EXPLORER_BASE}/transactions/${trimmed}`;
}

export function openTransactionExplorer(txid: string, network: ZcashNetwork = "mainnet"): void {
  const url = transactionExplorerUrl(txid, network);
  if (!url) return;
  window.open(url, "_blank", "noopener,noreferrer");
}
