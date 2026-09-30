
export interface UserFriendlyError {
  message: string;
  code?: string;
}

const ERROR_MAP: Array<{ pattern: RegExp | string; message: string; code?: string }> = [
  // Wallet & auth
  { pattern: /wrong password|incorrect password|invalid password/i, message: "Incorrect password. Please try again.", code: "AUTH_001" },
  { pattern: /password.*required|missing password/i, message: "Please enter your password.", code: "AUTH_002" },
  { pattern: /wallet.*locked|WALLET_LOCKED|not unlocked/i, message: "Wallet is locked. Please unlock with your password.", code: "WALLET_001" },
  { pattern: /wallet.*not found|no wallet found|WALLET_NOT_FOUND/i, message: "No wallet found. Create or restore a wallet first.", code: "WALLET_002" },
  { pattern: /wallet.*exist|WALLET_EXISTS/i, message: "A wallet already exists on this device.", code: "WALLET_005" },
  { pattern: /invalid mnemonic|bad mnemonic|invalid seed/i, message: "Invalid recovery phrase. Check the words and try again.", code: "WALLET_003" },
  { pattern: /mnemonic.*required/i, message: "Please enter your recovery phrase.", code: "WALLET_004" },
  // Send & balance
  {
    pattern: /Largest Ironwood note is|No single Ironwood note covers|multi-note spends are not supported/i,
    message:
      "This send needs more than any one Ironwood note. Use Max — that is the largest note minus fee. Combined balance is split across notes, and multi-note spends are not supported yet.",
    code: "SEND_IRONWOOD_NOTE",
  },
  { pattern: /insufficient|not enough|balance.*low/i, message: "Insufficient balance. Check your balance and try a smaller amount.", code: "SEND_001" },
  { pattern: /invalid address|bad address|invalid recipient/i, message: "Invalid recipient address. Please check and try again.", code: "SEND_002" },
  { pattern: /invalid amount|amount.*invalid/i, message: "Invalid amount. Enter a positive number.", code: "SEND_003" },
  { pattern: /ironwood notes only|hardware signing.*ironwood/i, message: "Ironwood (NU6.3) is active. Hardware wallet sends for Ironwood are not supported yet — use software Send in the app or `nozy send`.", code: "SEND_IRONWOOD" },
  { pattern: /orchard notes remain/i, message: "Orchard notes remain migrate if able.", code: "SEND_IRONWOOD" },
  {
    pattern: /safer migration|broadcast.*after tip sync|skip-broadcast-hygiene|need ≥?\s*120s|need >=?\s*120s/i,
    message:
      "Safer send wait: wait about 2 minutes after syncing to tip, then try again. This reduces leaking your send right after a sync burst.",
    code: "SEND_HYGIENE",
  },
  { pattern: /transaction.*fail|send.*fail/i, message: "Transaction failed. Check sync status and balance, then try again.", code: "SEND_004" },
  // Network & node
  { pattern: /connection refused|ECONNREFUSED|failed to connect/i, message: "Cannot connect to node. Check the node URL and that it's running.", code: "NET_001" },
  { pattern: /timeout|ETIMEDOUT|timed out/i, message: "Request timed out. The node may be slow or unreachable.", code: "NET_002" },
  { pattern: /network.*error|fetch failed/i, message: "Network error. Check your connection and try again.", code: "NET_003" },
  { pattern: /zebra|node.*unavailable/i, message: "Node is unavailable. Check Settings → Network and try again.", code: "NET_004" },
  // Sync & proving
  { pattern: /sync.*fail|scan.*fail/i, message: "Sync failed. Check your node connection and try again.", code: "SYNC_001" },
  { pattern: /proving|proof.*fail/i, message: "Proving failed. You may need to download proving parameters.", code: "PROVE_001" },
  // Backend / IPC
  { pattern: /command.*not found|unknown command|not found.*command/i, message: "This feature is not available in the current app build. Restart the desktop app after updating.", code: "BACKEND_001" },
  // NU7 vote
  {
    pattern: /witness root.*nc_root|store_witnesses|does not match stored round nc_root/i,
    message:
      "Vote note witnesses must be at the snapshot height. Re-run Vote step 1 (Export) — it rebuilds witnesses to the snapshot (can take several minutes), then Prepare again.",
    code: "VOTE_002",
  },
  {
    pattern: /van_leaf_position|van_witness|No confirmed delegation|missing VAN/i,
    message:
      "Delegation is not confirmed yet. Complete Vote step 5 (Submit delegation), wait for confirmation, then cast again.",
    code: "VOTE_001",
  },
  // Address book
  { pattern: /address.*already exists|duplicate.*name|contact.*already exist/i, message: "A contact with this name may already exist. Use a different name.", code: "CONTACT_001" },
  { pattern: /only shielded|must be.*shielded|shielded.*required/i, message: "Only shielded addresses (u1, utest1, or zs1) can be saved to contacts.", code: "CONTACT_002" },
  // Generic
  { pattern: /user denied|rejected|cancelled/i, message: "Action was cancelled.", code: "USER_001" },
  { pattern: /too many requests|rate limit/i, message: "Too many requests. Please wait a moment and try again.", code: "RATE_001" },
  // Crosslink (feature-net cTAZ)
  {
    pattern: /staking day is closed|CROSSLINK_STAKING_DAY_CLOSED/i,
    message:
      "Staking Day is closed. New stakes are allowed every 150 blocks for 70 blocks. Enable Force to try anyway, or use Retarget anytime.",
    code: "CROSSLINK_STAKING_DAY",
  },
  {
    pattern: /another stake is in progress|CROSSLINK_STAKE_BUSY|another stake in progress/i,
    message:
      "Crosslink node is still finishing a stake (one at a time). This can appear after a single click. Check Your bonds / GUI pending; wait, then retry only if nothing new landed.",
    code: "CROSSLINK_STAKE_BUSY",
  },
  {
    pattern: /failed to create staking transaction from notes/i,
    message:
      "Node wallet could not build that stake action. For Withdraw: Unbond first and wait until the bond shows as withdrawable. Also ensure the Crosslink GUI wallet has spendable cTAZ for fees.",
    code: "CROSSLINK_NOTES",
  },
  {
    pattern: /finalizer identity must be 64 hex/i,
    message: "Finalizer must be the full 64-character hex key (use Show full keys or copy from the GUI).",
    code: "CROSSLINK_FINALIZER",
  },
];

function normalizeError(error: unknown): string {
  if (error == null) return "";
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (typeof error === "object") {
    const o = error as Record<string, unknown>;
    if (typeof o.message === "string") return o.message;
    const payload = o.payload;
    if (typeof payload === "string") return payload;
    if (payload && typeof payload === "object") {
      const p = payload as Record<string, unknown>;
      if (typeof p.message === "string") return p.message;
    }
  }
  try {
    return String(JSON.stringify(error));
  } catch {
    return String(error);
  }
}

/**
 * Returns a user-friendly error message (and optional code) for display in the UI.
 */
export function getUserFriendlyMessage(error: unknown, fallback: string): UserFriendlyError {
  const raw = normalizeError(error).trim();
  if (!raw) return { message: fallback };

  const lower = raw.toLowerCase();
  for (const { pattern, message, code } of ERROR_MAP) {
    const matches = typeof pattern === "string" ? lower.includes(pattern.toLowerCase()) : pattern.test(raw);
    if (matches) return { message, code };
  }

  const looksLikeStackTrace = raw.includes(" at ") || raw.includes("Error:");
  if (!looksLikeStackTrace) {
    if (raw.length <= 320) {
      return { message: raw };
    }
    return { message: `${raw.slice(0, 317)}…` };
  }
  return { message: fallback };
}

/**
 * Formats an error for toast or inline display.
 */
export function formatErrorForDisplay(error: unknown, fallback: string, options?: { showCode?: boolean }): string {
  const { message, code } = getUserFriendlyMessage(error, fallback);
  const showCode = options?.showCode ?? (typeof import.meta !== "undefined" && import.meta.env?.DEV);
  if (showCode && code) return `${message} (${code})`;
  return message;
}
