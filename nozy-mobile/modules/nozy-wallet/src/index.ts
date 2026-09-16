/**
 * JS bridge for on-device `libnozy_ffi` (UniFFI) via the Expo module `NozyFfi`.
 *
 * Keys stay on the phone. Compact sync uses lightwalletd.
 */

import { NativeModules, Platform } from "react-native";

export type SaplingStatusNative = {
  unspent_notes: number;
  with_rseed: number;
  ready_to_shield: number;
  unspent_zatoshis: number;
  unspent_zec: number;
  fee_zatoshis: number;
  fee_zec: number;
  has_legacy_balance: boolean;
  message: string;
};

export type OrchardScanNative = {
  blocks_scanned: number;
  actions_seen: number;
  orchard_actions_seen?: number;
  ironwood_actions_seen?: number;
  notes_discovered: number;
  orchard_notes_discovered?: number;
  ironwood_notes_discovered?: number;
  notes_marked_spent: number;
  range_start: number;
  range_end: number;
  unspent_zatoshis: number;
  unspent_zec: number;
  unspent_notes: number;
  message: string;
};

export type OrchardReceivedNoteNative = {
  txid: string;
  block_height: number;
  value_zec: number;
  spent: boolean;
  pool: string;
};

export type OrchardStatusNative = {
  unspent_zatoshis: number;
  unspent_zec: number;
  unspent_notes: number;
  notes: OrchardReceivedNoteNative[];
  message: string;
};

export type SaplingScanNative = {
  blocks_scanned: number;
  outputs_seen: number;
  notes_discovered: number;
  notes_marked_spent: number;
  range_start: number;
  range_end: number;
  unspent_zatoshis: number;
  unspent_notes: number;
  message: string;
};

export type SaplingShieldNative = {
  dry_run: boolean;
  broadcast: boolean;
  txid: string | null;
  shielded_value_zatoshis: number | null;
  fee_zatoshis: number;
  expiry_height: number | null;
  candidate_notes: number;
  candidate_zatoshis: number;
  message: string;
};

export type VoteCalendarNative = {
  snapshot_utc: string;
  vote_start_utc: string;
  vote_end_utc: string;
  forum_url: string;
  tally_url: string;
  message: string;
};

export type VoteNotesExportNative = {
  format: string;
  network: string;
  note_count: number;
  total_value_zat: number;
  seed_fingerprint_hex: string;
  notes_json: string;
  message: string;
};

export type VoteDelegationSigNative = {
  format: string;
  round_id: string;
  bundle_index: number;
  sighash_hex: string;
  spend_auth_sig_hex: string;
  sig_json: string;
  message: string;
};

export type WalletPathsNative = {
  walletDataDir: string;
  compactDbPath: string;
};

export type LwdInfoNative = {
  version: string;
  chain_name: string;
  block_height: number;
  estimated_height: number;
};

export type UnifiedAddressNative = {
  address: string;
  network: string;
};

type NozyFfiNative = {
  nativeLibReady?: () => boolean;
  walletPaths?: () => WalletPathsNative;
  saplingStatus?: (walletDataDir: string) => Promise<SaplingStatusNative>;
  saplingScan?: (
    mnemonic: string,
    walletDataDir: string,
    compactDbPath: string,
    startFloor: number | null,
    full: boolean,
  ) => Promise<SaplingScanNative>;
  orchardScan?: (
    mnemonic: string,
    walletDataDir: string,
    compactDbPath: string,
    startFloor: number | null,
    maxAccount: number,
    full: boolean,
  ) => Promise<OrchardScanNative>;
  orchardStatus?: (walletDataDir: string) => Promise<OrchardStatusNative>;
  saplingShield?: (
    mnemonic: string,
    walletDataDir: string,
    compactDbPath: string,
    zebraUrl: string,
    lightwalletdUrl: string,
    dryRun: boolean,
    noBroadcast: boolean,
  ) => Promise<SaplingShieldNative>;
  generateMnemonic?: () => Promise<string>;
  validateMnemonic?: (mnemonic: string) => Promise<void>;
  orchardUnifiedAddress?: (
    mnemonic: string,
    account: number,
  ) => Promise<UnifiedAddressNative>;
  lwdGetInfo?: (lightwalletdUrl: string) => Promise<LwdInfoNative>;
  lwdSyncCompactToTip?: (
    lightwalletdUrl: string,
    compactDbPath: string,
    startFloor: number | null,
  ) => Promise<number>;
  lwdGetLatestTreeState?: (lightwalletdUrl: string) => Promise<{
    network: string;
    height: number;
    hash: string;
    time: number;
    sapling_tree: string;
    orchard_tree: string;
  }>;
  lwdSendTransaction?: (
    lightwalletdUrl: string,
    rawTxHex: string,
  ) => Promise<void>;
  voteCalendarInfo?: () => Promise<VoteCalendarNative> | VoteCalendarNative;
  voteExportNotes?: (
    mnemonic: string,
    walletDataDir: string,
    network: string,
  ) => Promise<VoteNotesExportNative>;
  voteSignDelegation?: (
    mnemonic: string,
    requestJson: string,
  ) => Promise<VoteDelegationSigNative>;
  lockWallet?: () => void;
};

function loadExpoModule(): NozyFfiNative | null {
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const { requireNativeModule } = require("expo-modules-core") as {
      requireNativeModule: (name: string) => NozyFfiNative;
    };
    const mod = requireNativeModule("NozyFfi");
    return mod;
  } catch {
    return null;
  }
}

function native(): NozyFfiNative | null {
  const expo = loadExpoModule();
  if (expo) return expo;
  const legacy = NativeModules.NozyFfi as NozyFfiNative | undefined;
  return legacy ?? null;
}

export function isNozyWalletNativeAvailable(): boolean {
  if (Platform.OS !== "android" && Platform.OS !== "ios") {
    return false;
  }
  const n = native();
  if (!n) return false;
  try {
    if (typeof n.nativeLibReady === "function") {
      return n.nativeLibReady() === true;
    }
  } catch {
    return false;
  }
  return typeof n.generateMnemonic === "function";
}

/** Clears in-memory on-device session (if the native module tracks one). */
export function lockOnDeviceWallet(): void {
  try {
    native()?.lockWallet?.();
  } catch {
    // Expo Go / missing native module
  }
}

function requireNative(): NozyFfiNative {
  const n = native();
  if (!n) {
    throw new Error(
      "On-device wallet requires a native build with libnozy_ffi (see nozy-ffi/README.md).",
    );
  }
  return n;
}

export function walletPaths(): WalletPathsNative {
  const n = requireNative();
  if (typeof n.walletPaths !== "function") {
    throw new Error("walletPaths is not available on this native module.");
  }
  return n.walletPaths();
}

export async function generateMnemonic(): Promise<string> {
  const n = requireNative();
  if (!n.generateMnemonic) {
    throw new Error("generateMnemonic requires libnozy_ffi.");
  }
  return n.generateMnemonic();
}

export async function validateMnemonic(mnemonic: string): Promise<void> {
  const n = requireNative();
  if (!n.validateMnemonic) {
    throw new Error("validateMnemonic requires libnozy_ffi.");
  }
  await n.validateMnemonic(mnemonic);
}

export async function lwdGetInfo(
  lightwalletdUrl: string,
): Promise<LwdInfoNative> {
  const n = requireNative();
  if (!n.lwdGetInfo) {
    throw new Error("lwdGetInfo requires libnozy_ffi.");
  }
  return n.lwdGetInfo(lightwalletdUrl);
}

export async function lwdSendTransaction(params: {
  lightwalletdUrl?: string;
  rawTxHex: string;
}): Promise<void> {
  const n = requireNative();
  if (!n.lwdSendTransaction) {
    throw new Error("lwdSendTransaction requires libnozy_ffi.");
  }
  const url =
    params.lightwalletdUrl?.trim() || "https://lwd.nozywallet.org:443";
  await n.lwdSendTransaction(url, params.rawTxHex);
}

export async function saplingStatus(
  walletDataDir: string,
): Promise<SaplingStatusNative> {
  const n = requireNative();
  if (!n.saplingStatus) {
    throw new Error(
      "On-device Sapling requires a native build with libnozy_ffi (see nozy-ffi/README.md).",
    );
  }
  return n.saplingStatus(walletDataDir);
}

export async function saplingScan(params: {
  mnemonic: string;
  walletDataDir: string;
  compactDbPath: string;
  startFloor?: number | null;
  full?: boolean;
}): Promise<SaplingScanNative> {
  const n = requireNative();
  if (!n.saplingScan) {
    throw new Error(
      "On-device Sapling requires a native build with libnozy_ffi (see nozy-ffi/README.md).",
    );
  }
  return n.saplingScan(
    params.mnemonic,
    params.walletDataDir,
    params.compactDbPath,
    params.startFloor ?? null,
    params.full ?? false,
  );
}

export async function orchardScan(params: {
  mnemonic: string;
  walletDataDir: string;
  compactDbPath: string;
  startFloor?: number | null;
  maxAccount?: number;
  full?: boolean;
}): Promise<OrchardScanNative> {
  const n = requireNative();
  if (!n.orchardScan) {
    throw new Error(
      "On-device Orchard scan requires a native rebuild with libnozy_ffi.",
    );
  }
  return n.orchardScan(
    params.mnemonic,
    params.walletDataDir,
    params.compactDbPath,
    params.startFloor ?? null,
    params.maxAccount ?? 0,
    params.full ?? false,
  );
}

export async function orchardStatus(
  walletDataDir: string,
): Promise<OrchardStatusNative> {
  const n = requireNative();
  if (!n.orchardStatus) {
    throw new Error(
      "On-device Orchard status requires a native rebuild with libnozy_ffi.",
    );
  }
  return n.orchardStatus(walletDataDir);
}

export async function saplingShield(params: {
  mnemonic: string;
  walletDataDir: string;
  compactDbPath: string;
  zebraUrl: string;
  lightwalletdUrl: string;
  dryRun?: boolean;
  noBroadcast?: boolean;
}): Promise<SaplingShieldNative> {
  const n = requireNative();
  if (!n.saplingShield) {
    throw new Error(
      "On-device Sapling requires a native build with libnozy_ffi (see nozy-ffi/README.md).",
    );
  }
  return n.saplingShield(
    params.mnemonic,
    params.walletDataDir,
    params.compactDbPath,
    params.zebraUrl,
    params.lightwalletdUrl,
    params.dryRun ?? false,
    params.noBroadcast ?? false,
  );
}

export async function voteCalendarInfo(): Promise<VoteCalendarNative> {
  const n = native();
  if (!n?.voteCalendarInfo) {
    // Static fallback when native module not rebuilt yet
    return {
      snapshot_utc: "2026-08-24T19:00:00Z",
      vote_start_utc: "2026-08-25T00:00:00Z",
      vote_end_utc: "2026-09-14T19:00:00Z",
      forum_url: "https://forum.zcashcommunity.com/t/nu7-coinholder-vote/56912",
      tally_url: "https://tally.valargroup.org",
      message:
        "Eligible weight = spendable Ironwood notes at snapshot. Prepare/cast on desktop or nozy-vote.",
    };
  }
  return await n.voteCalendarInfo();
}

export async function voteExportNotes(params: {
  mnemonic: string;
  walletDataDir: string;
  network?: string;
}): Promise<VoteNotesExportNative> {
  const n = native();
  if (!n?.voteExportNotes) {
    throw new Error(
      "On-device vote export requires a native build with libnozy_ffi (rebuild nozy-ffi + bindgen).",
    );
  }
  return n.voteExportNotes(
    params.mnemonic,
    params.walletDataDir,
    params.network ?? "mainnet",
  );
}

export async function voteSignDelegation(params: {
  mnemonic: string;
  requestJson: string;
}): Promise<VoteDelegationSigNative> {
  const n = native();
  if (!n?.voteSignDelegation) {
    throw new Error(
      "On-device vote sign requires a native build with libnozy_ffi (rebuild nozy-ffi + bindgen).",
    );
  }
  return n.voteSignDelegation(params.mnemonic, params.requestJson);
}

export async function orchardUnifiedAddress(
  mnemonic: string,
  account = 0,
): Promise<UnifiedAddressNative> {
  const n = requireNative();
  if (!n.orchardUnifiedAddress) {
    throw new Error(
      "On-device address requires a native build with libnozy_ffi (rebuild nozy-ffi).",
    );
  }
  return n.orchardUnifiedAddress(mnemonic, account);
}

export async function lwdSyncCompactToTip(params: {
  lightwalletdUrl?: string;
  compactDbPath: string;
  startFloor?: number | null;
}): Promise<number> {
  const n = requireNative();
  if (!n.lwdSyncCompactToTip) {
    throw new Error(
      "On-device compact sync requires a native build with libnozy_ffi.",
    );
  }
  const url =
    params.lightwalletdUrl?.trim() || "https://lwd.nozywallet.org:443";
  return n.lwdSyncCompactToTip(
    url,
    params.compactDbPath,
    params.startFloor ?? null,
  );
}
