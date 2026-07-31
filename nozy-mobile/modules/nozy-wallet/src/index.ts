/**
 * JS bridge for on-device `libnozy_ffi` (UniFFI).
 *
 * After `cargo ndk` + `uniffi-bindgen` (see nozy-ffi/README.md), wire generated
 * Kotlin into NativeModules.NozyFfi. Until then, native APIs are unavailable
 * and the app stays on companion HTTP.
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

type NozyFfiNative = {
  saplingStatus: (walletDataDir: string) => Promise<SaplingStatusNative>;
  saplingScan: (
    mnemonic: string,
    walletDataDir: string,
    compactDbPath: string,
    startFloor: number | null,
    full: boolean,
  ) => Promise<SaplingScanNative>;
  saplingShield: (
    mnemonic: string,
    walletDataDir: string,
    compactDbPath: string,
    zebraUrl: string,
    lightwalletdUrl: string,
    dryRun: boolean,
    noBroadcast: boolean,
  ) => Promise<SaplingShieldNative>;
  lockWallet?: () => void;
};

function native(): NozyFfiNative | null {
  const mod = NativeModules.NozyFfi as NozyFfiNative | undefined;
  if (!mod || typeof mod.saplingStatus !== "function") {
    return null;
  }
  return mod;
}

export function isNozyWalletNativeAvailable(): boolean {
  if (Platform.OS !== "android" && Platform.OS !== "ios") {
    return false;
  }
  return native() !== null;
}

/** Clears in-memory on-device session (if the native module tracks one). */
export function lockOnDeviceWallet(): void {
  try {
    native()?.lockWallet?.();
  } catch {
    // Expo Go / missing native module
  }
}

export async function saplingStatus(
  walletDataDir: string,
): Promise<SaplingStatusNative> {
  const n = native();
  if (!n) {
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
  const n = native();
  if (!n) {
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

export async function saplingShield(params: {
  mnemonic: string;
  walletDataDir: string;
  compactDbPath: string;
  zebraUrl: string;
  lightwalletdUrl: string;
  dryRun?: boolean;
  noBroadcast?: boolean;
}): Promise<SaplingShieldNative> {
  const n = native();
  if (!n) {
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
