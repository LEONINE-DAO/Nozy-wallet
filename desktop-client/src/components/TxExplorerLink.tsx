import { openTransactionExplorer, transactionExplorerUrl, type ZcashNetwork } from "../utils/explorer";

function ExternalLinkIcon({ className = "" }: { className?: string }) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
    >
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15 3 21 3 21 9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </svg>
  );
}

/** High-contrast explorer link styles (Tailwind preflight hides default link styling). */
const EXPLORER_LINK =
  "inline-flex items-center gap-1.5 font-semibold text-[#9a6500] dark:text-[#f0a113] underline decoration-[#c4820a] decoration-2 underline-offset-2 hover:text-[#7a4f00] dark:hover:text-[#ffc04d] transition-colors";

const EXPLORER_PILL =
  "inline-flex items-center gap-1 rounded-md border border-[#f0a113]/40 bg-[#f0a113]/12 px-2 py-0.5 text-xs font-semibold text-[#9a6500] dark:text-[#f0a113] hover:bg-[#f0a113]/20 hover:border-[#f0a113]/60 transition-colors";

function handleExplorerClick(e: React.MouseEvent<HTMLAnchorElement>, txid: string, network: ZcashNetwork) {
  // Tauri webviews sometimes ignore target="_blank"; window.open is the reliable path.
  e.preventDefault();
  openTransactionExplorer(txid, network);
}

interface TxExplorerLinkProps {
  txid: string;
  network?: ZcashNetwork;
  className?: string;
  label?: string;
  variant?: "link" | "pill";
}

/** Link styled as the TXID itself (for receipts and detail views). */
export function TxIdLink({
  txid,
  network = "mainnet",
  className = "",
}: {
  txid: string;
  network?: ZcashNetwork;
  className?: string;
}) {
  const url = transactionExplorerUrl(txid, network);
  if (!url) {
    return <span className={`font-mono text-sm text-gray-900 dark:text-gray-100 break-all ${className}`}>{txid}</span>;
  }

  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      title="View on mainnet.zcashexplorer.app"
      onClick={(e) => handleExplorerClick(e, txid, network)}
      className={`font-mono text-sm break-all cursor-pointer ${EXPLORER_LINK} ${className}`}
    >
      {txid}
      <ExternalLinkIcon className="shrink-0 opacity-80" />
    </a>
  );
}

export function TxExplorerLink({
  txid,
  network = "mainnet",
  className = "",
  label = "View on Zcash Explorer",
  variant = "link",
}: TxExplorerLinkProps) {
  const url = transactionExplorerUrl(txid, network);
  if (!url) return null;

  const style = variant === "pill" ? EXPLORER_PILL : EXPLORER_LINK;

  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      title="Opens mainnet.zcashexplorer.app in your browser"
      onClick={(e) => handleExplorerClick(e, txid, network)}
      className={`${style} ${className}`}
    >
      {label}
      <ExternalLinkIcon className="shrink-0 opacity-80" />
    </a>
  );
}

interface TransactionIdDetailProps {
  txid: string;
  network?: ZcashNetwork;
}

export function TransactionIdDetail({ txid, network = "mainnet" }: TransactionIdDetailProps) {
  return (
    <div className="rounded-xl border border-[#f0a113]/35 bg-[#f0a113]/8 p-3 space-y-2">
      <p className="text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wide">
        Transaction ID
      </p>
      <TxIdLink txid={txid} network={network} />
      <TxExplorerLink
        txid={txid}
        network={network}
        label="View on mainnet.zcashexplorer.app"
        variant="pill"
        className="mt-1"
      />
    </div>
  );
}
