
export interface UserFriendlyError {
  message: string;
  code?: string;
}

const ERROR_MAP: Array<{ pattern: RegExp | string; message: string; code?: string }> = [
  // Wallet & auth
  { pattern: /wrong password|incorrect password|invalid password/i, message: "Incorrect password. Please try again.", code: "AUTH_001" },
  { pattern: /password.*required|missing password/i, message: "Please enter your password.", code: "AUTH_002" },
  { pattern: /wallet.*locked|not unlocked/i, message: "Wallet is locked. Please unlock with your password.", code: "WALLET_001" },
  { pattern: /wallet.*not found|wallet.*exist|no wallet/i, message: "No wallet found. Create or restore a wallet first.", code: "WALLET_002" },
  { pattern: /invalid mnemonic|bad mnemonic|invalid seed/i, message: "Invalid recovery phrase. Check the words and try again.", code: "WALLET_003" },
  { pattern: /mnemonic.*required/i, message: "Please enter your recovery phrase.", code: "WALLET_004" },
  // Send & balance
  { pattern: /insufficient|not enough|balance.*low/i, message: "Insufficient balance. Check your balance and try a smaller amount.", code: "SEND_001" },
  { pattern: /invalid address|bad address|invalid recipient/i, message: "Invalid recipient address. Please check and try again.", code: "SEND_002" },
  { pattern: /invalid amount|amount.*invalid/i, message: "Invalid amount. Enter a positive number.", code: "SEND_003" },
  { pattern: /transaction.*fail|send.*fail/i, message: "Transaction failed. Check your password and balance, then try again.", code: "SEND_004" },
  // Network & node
  { pattern: /connection refused|ECONNREFUSED|failed to connect/i, message: "Cannot connect to node. Check the node URL and that it's running.", code: "NET_001" },
  { pattern: /timeout|ETIMEDOUT|timed out/i, message: "Request timed out. The node may be slow or unreachable.", code: "NET_002" },
  { pattern: /network.*error|fetch failed/i, message: "Network error. Check your connection and try again.", code: "NET_003" },
  { pattern: /zebra|node.*unavailable/i, message: "Node is unavailable. Check Settings → Network and try again.", code: "NET_004" },
  // Sync & proving
  { pattern: /sync.*fail|scan.*fail/i, message: "Sync failed. Check your node connection and try again.", code: "SYNC_001" },
  { pattern: /proving|proof.*fail/i, message: "Proving failed. You may need to download proving parameters.", code: "PROVE_001" },
  // Address book
  { pattern: /address.*book|contact.*exist|duplicate.*name/i, message: "A contact with this name may already exist. Use a different name.", code: "CONTACT_001" },
  { pattern: /only shielded|must be.*shielded|shielded.*required/i, message: "Only shielded addresses (u1 or zs1) can be saved to contacts.", code: "CONTACT_002" },
  // Generic
  { pattern: /user denied|rejected|cancelled/i, message: "Action was cancelled.", code: "USER_001" },
  { pattern: /too many requests|rate limit/i, message: "Too many requests. Please wait a moment and try again.", code: "RATE_001" },
];

function normalizeError(error: unknown): string {
  if (error == null) return "";
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (typeof (error as { message?: string }).message === "string") return (error as { message: string }).message;
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

  if (raw.length <= 120 && !raw.includes("Error:") && !raw.includes(" at ")) {
    return { message: raw };
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
