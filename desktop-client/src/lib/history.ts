export interface HistoryTx {

  id: string;

  type: "sent" | "received" | "change";

  amount: number;

  date: string;

  address: string;

  status: string;

  memo?: string;

  blockHeight?: number;

  confirmations?: number;

  isChange?: boolean;

}



const MIN_VALID_HISTORY_MS = Date.parse("2016-10-01T00:00:00.000Z");



function parseHistoryDateMs(raw: unknown): number | null {

  if (typeof raw === "string") {

    const ms = Date.parse(raw);

    return Number.isNaN(ms) ? null : ms;

  }

  if (raw != null && typeof raw === "object" && "secs_since_epoch" in raw) {

    return (raw as { secs_since_epoch: number }).secs_since_epoch * 1000;

  }

  if (raw != null && typeof raw === "number") {

    return raw * 1000;

  }

  return null;

}



function isoDateTimeFromMs(ms: number | null): string {

  if (ms == null || ms < MIN_VALID_HISTORY_MS) return "";

  return new Date(ms).toISOString();

}



/** User-facing date label; falls back to block height when chain time is unavailable. */

export function formatHistoryDate(tx: Pick<HistoryTx, "date" | "blockHeight">): string {

  if (tx.date) {

    const ms = Date.parse(tx.date);

    if (!Number.isNaN(ms) && ms >= MIN_VALID_HISTORY_MS) {

      return new Date(ms).toLocaleDateString(undefined, { dateStyle: "medium" });

    }

  }

  if (tx.blockHeight != null) {

    return `Block ${tx.blockHeight.toLocaleString()}`;

  }

  return "—";

}



/** Format for detail modals (full locale string or block height). */

export function formatHistoryDetailDate(tx: Pick<HistoryTx, "date" | "blockHeight">): string {

  if (tx.date) {

    const ms = Date.parse(tx.date);

    if (!Number.isNaN(ms) && ms >= MIN_VALID_HISTORY_MS) {

      return new Date(ms).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });

    }

  }

  if (tx.blockHeight != null) {

    return `Block ${tx.blockHeight.toLocaleString()}`;

  }

  return "—";

}



function normalizeStatus(raw: unknown, type: HistoryTx["type"]): string {

  const statusRaw =

    raw != null ? String(raw).toLowerCase().replace(/\s/g, "_") : "";

  if (

    statusRaw === "confirmed" ||

    statusRaw === "pending" ||

    statusRaw === "failed" ||

    statusRaw === "expired"

  ) {

    return statusRaw;

  }

  if (type === "received" || type === "change") {

    return "confirmed";

  }

  return statusRaw || "unknown";

}



function normalizeType(raw: Record<string, unknown>): HistoryTx["type"] {

  if (raw.is_change === true) {

    return "change";

  }

  const typeRaw = String(raw.transaction_type ?? raw.type ?? "sent").toLowerCase();

  if (typeRaw === "received") return "received";

  if (typeRaw === "change") return "change";

  if (typeRaw === "sent") return "sent";

  return "sent";

}



/** Normalize API / Tauri history JSON (sent + received from `collect_wallet_transaction_views`). */

export function normalizeHistoryTx(raw: Record<string, unknown>): HistoryTx {

  const txid = String(raw.txid ?? raw.id ?? "");

  const amountZec =

    typeof raw.amount_zec === "number"

      ? raw.amount_zec

      : raw.amount_zatoshis != null

        ? Number(raw.amount_zatoshis) / 100_000_000

        : 0;



  const type = normalizeType(raw);

  const recipient = String(raw.recipient_address ?? raw.recipient ?? "");

  const depositAddress =

    typeof raw.deposit_address === "string" ? raw.deposit_address : "";



  const status = normalizeStatus(raw.status, type);



  const dateRaw = raw.broadcast_at ?? raw.created_at ?? raw.date ?? raw.block_time;

  const date = isoDateTimeFromMs(parseHistoryDateMs(dateRaw));



  const memo = typeof raw.memo === "string" ? raw.memo : undefined;



  let address = recipient || depositAddress;

  if (!address) {

    if (type === "change") {

      address = "Your wallet (change from send)";

    } else if (type === "received") {

      address = "Shielded deposit";

    } else if (txid) {

      address = `${txid.slice(0, 8)}…${txid.slice(-4)}`;

    } else {

      address = "—";

    }

  }



  return {

    id: txid,

    type,

    amount: amountZec,

    date,

    address,

    status,

    memo,

    blockHeight: typeof raw.block_height === "number" ? raw.block_height : undefined,

    confirmations: typeof raw.confirmations === "number" ? raw.confirmations : undefined,

    isChange: type === "change",

  };

}



/** Sort history rows newest-first using on-chain block height, then date. */

export function sortHistoryNewestFirst(txs: HistoryTx[]): HistoryTx[] {

  return [...txs].sort((a, b) => {

    const ah = a.blockHeight ?? 0;

    const bh = b.blockHeight ?? 0;

    if (bh !== ah) return bh - ah;

    const da = a.date ? new Date(a.date).getTime() : 0;

    const db = b.date ? new Date(b.date).getTime() : 0;

    if (db !== da) return db - da;

    return b.id.localeCompare(a.id);

  });

}



/** Compact address for home activity cards (matches original mock-style truncation). */

export function shortenAddress(value: string, head = 8, tail = 4): string {

  const trimmed = value.trim();

  if (!trimmed) return "—";

  if (trimmed.length <= head + tail + 3) return trimmed;

  return `${trimmed.slice(0, head)}…${trimmed.slice(-tail)}`;

}



export function historyTypeLabel(tx: Pick<HistoryTx, "type" | "isChange">): string {

  if (tx.type === "change" || tx.isChange) return "Change (your send)";

  if (tx.type === "received") return "Received";

  return "Sent";

}



export function historyAmountPrefix(tx: Pick<HistoryTx, "type" | "isChange">): string {

  if (tx.type === "change" || tx.isChange) return "";

  if (tx.type === "received") return "+";

  return "-";

}


