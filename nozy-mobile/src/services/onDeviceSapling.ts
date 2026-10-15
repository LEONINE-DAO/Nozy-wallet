import AsyncStorage from "@react-native-async-storage/async-storage";
import {
  lwdGetInfo,
  saplingScan,
  saplingShield,
  saplingStatus,
  walletPaths,
  type SaplingStatusNative,
} from "nozy-wallet";
import {
  ONDEVICE_BIRTHDAY_KEY,
  ONDEVICE_COMPACT_DB_KEY,
  ONDEVICE_DATA_DIR_KEY,
  ONDEVICE_LWD_URL_KEY,
  ONDEVICE_MNEMONIC_KEY,
  ONDEVICE_WALLET_KIND_KEY,
  ONDEVICE_ZEBRA_URL_KEY,
} from "../components/settings/OnDeviceWalletSettings";
import {
  NOZY_PUBLIC_LWD_URL,
  localNodeBlockReason,
  syncServerBlockReason,
} from "../lib/connectionPresets";
import { getSecret } from "../lib/secretStorage";
import { NU5_ORCHARD_MAINNET } from "../lib/zcashHeights";
import type { SaplingStatusResponse } from "../types";

export type OnDevicePaths = {
  mnemonic: string;
  walletDataDir: string;
  compactDbPath: string;
  zebraUrl: string;
  lightwalletdUrl: string;
};

export async function loadOnDevicePaths(): Promise<OnDevicePaths | null> {
  const [mnemonic, storedDir, storedDb, zebraUrl, lightwalletdUrl] =
    await Promise.all([
      getSecret(ONDEVICE_MNEMONIC_KEY),
      AsyncStorage.getItem(ONDEVICE_DATA_DIR_KEY),
      AsyncStorage.getItem(ONDEVICE_COMPACT_DB_KEY),
      AsyncStorage.getItem(ONDEVICE_ZEBRA_URL_KEY),
      AsyncStorage.getItem(ONDEVICE_LWD_URL_KEY),
    ]);
  if (!mnemonic?.trim()) return null;

  let walletDataDir = storedDir?.trim() ?? "";
  let compactDbPath = storedDb?.trim() ?? "";
  if (!walletDataDir || !compactDbPath) {
    try {
      const native = walletPaths();
      walletDataDir = walletDataDir || native.walletDataDir;
      compactDbPath = compactDbPath || native.compactDbPath;
    } catch {
      // Address reveal only needs the seed. Sync can fill paths later.
    }
  }
  if (
    walletDataDir &&
    compactDbPath &&
    (walletDataDir !== storedDir?.trim() || compactDbPath !== storedDb?.trim())
  ) {
    await AsyncStorage.multiSet([
      [ONDEVICE_DATA_DIR_KEY, walletDataDir],
      [ONDEVICE_COMPACT_DB_KEY, compactDbPath],
    ]);
  }
  let zebra = (zebraUrl ?? "").trim();
  if (zebra && localNodeBlockReason(zebra)) {
    zebra = "";
    await AsyncStorage.removeItem(ONDEVICE_ZEBRA_URL_KEY);
  }
  let lwd = (lightwalletdUrl ?? "").trim();
  if (!lwd || syncServerBlockReason(lwd)) {
    lwd = NOZY_PUBLIC_LWD_URL;
    await AsyncStorage.setItem(ONDEVICE_LWD_URL_KEY, lwd);
  }
  return {
    mnemonic: mnemonic.trim(),
    walletDataDir,
    compactDbPath,
    zebraUrl: zebra,
    lightwalletdUrl: lwd,
  };
}

export async function persistOnDeviceBirthday(height: number): Promise<void> {
  await AsyncStorage.setItem(ONDEVICE_BIRTHDAY_KEY, String(height));
}

async function birthdayFromLwdTip(): Promise<number> {
  const storedLwd = (await AsyncStorage.getItem(ONDEVICE_LWD_URL_KEY))?.trim() || "";
  const lwd = !storedLwd || syncServerBlockReason(storedLwd) ? NOZY_PUBLIC_LWD_URL : storedLwd;
  let birthday = NU5_ORCHARD_MAINNET;
  try {
    const info = await lwdGetInfo(lwd);
    const h = Number(info.block_height) || Number(info.estimated_height) || 0;
    if (h > birthday) birthday = Math.floor(h);
  } catch {
    // LWD unreachable — NU5 still avoids genesis (height 1).
  }
  return birthday;
}

/**
 * Compact-sync start height.
 * Restore wallets keep the stored birthday (NU5 unless the user set one).
 * Created / legacy wallets that only have NU5 or height 1 use current LWD tip
 * so a 6 GB emulator cannot download 1.8M compact blocks.
 */
export async function loadOnDeviceBirthday(): Promise<number> {
  const [raw, kind] = await Promise.all([
    AsyncStorage.getItem(ONDEVICE_BIRTHDAY_KEY),
    AsyncStorage.getItem(ONDEVICE_WALLET_KIND_KEY),
  ]);
  const n = raw ? parseInt(raw, 10) : NaN;
  const isRestore = kind === "restore";
  if (isRestore && Number.isFinite(n) && n >= NU5_ORCHARD_MAINNET) {
    return n;
  }
  if (!isRestore && Number.isFinite(n) && n > NU5_ORCHARD_MAINNET) {
    return n;
  }
  const birthday = await birthdayFromLwdTip();
  await persistOnDeviceBirthday(birthday);
  if (!kind) {
    await AsyncStorage.setItem(ONDEVICE_WALLET_KIND_KEY, "create");
  }
  return birthday;
}

export function mapNativeStatus(s: SaplingStatusNative): SaplingStatusResponse {
  return {
    unspent_notes: s.unspent_notes,
    with_rseed: s.with_rseed,
    ready_to_shield: s.ready_to_shield,
    unspent_zatoshis: s.unspent_zatoshis,
    unspent_zec: s.unspent_zec,
    fee_zatoshis: s.fee_zatoshis,
    fee_zec: s.fee_zec,
    has_legacy_balance: s.has_legacy_balance,
    message: s.message,
  };
}

export async function onDeviceSaplingStatus(): Promise<SaplingStatusResponse> {
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error(
      "Configure on-device mnemonic and data paths in Settings → On-device wallet.",
    );
  }
  return mapNativeStatus(await saplingStatus(paths.walletDataDir));
}

export async function onDeviceMoveLegacy(): Promise<string> {
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error(
      "Configure on-device mnemonic and data paths in Settings → On-device wallet.",
    );
  }
  const startFloor = await loadOnDeviceBirthday();
  await saplingScan({
    mnemonic: paths.mnemonic,
    walletDataDir: paths.walletDataDir,
    compactDbPath: paths.compactDbPath,
    startFloor,
  });
  const res = await saplingShield({
    mnemonic: paths.mnemonic,
    walletDataDir: paths.walletDataDir,
    compactDbPath: paths.compactDbPath,
    zebraUrl: paths.zebraUrl,
    lightwalletdUrl: paths.lightwalletdUrl,
  });
  return res.message;
}
