import AsyncStorage from "@react-native-async-storage/async-storage";
import {
  saplingScan,
  saplingShield,
  saplingStatus,
  type SaplingStatusNative,
} from "nozy-wallet";
import {
  ONDEVICE_COMPACT_DB_KEY,
  ONDEVICE_DATA_DIR_KEY,
  ONDEVICE_LWD_URL_KEY,
  ONDEVICE_MNEMONIC_KEY,
  ONDEVICE_ZEBRA_URL_KEY,
} from "../components/settings/OnDeviceWalletSettings";
import type { SaplingStatusResponse } from "../types";

export type OnDevicePaths = {
  mnemonic: string;
  walletDataDir: string;
  compactDbPath: string;
  zebraUrl: string;
  lightwalletdUrl: string;
};

export async function loadOnDevicePaths(): Promise<OnDevicePaths | null> {
  const [mnemonic, walletDataDir, compactDbPath, zebraUrl, lightwalletdUrl] =
    await Promise.all([
      AsyncStorage.getItem(ONDEVICE_MNEMONIC_KEY),
      AsyncStorage.getItem(ONDEVICE_DATA_DIR_KEY),
      AsyncStorage.getItem(ONDEVICE_COMPACT_DB_KEY),
      AsyncStorage.getItem(ONDEVICE_ZEBRA_URL_KEY),
      AsyncStorage.getItem(ONDEVICE_LWD_URL_KEY),
    ]);
  if (!mnemonic?.trim() || !walletDataDir?.trim() || !compactDbPath?.trim()) {
    return null;
  }
  return {
    mnemonic: mnemonic.trim(),
    walletDataDir: walletDataDir.trim(),
    compactDbPath: compactDbPath.trim(),
    zebraUrl: (zebraUrl ?? "").trim(),
    lightwalletdUrl: (lightwalletdUrl ?? "").trim(),
  };
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
  await saplingScan({
    mnemonic: paths.mnemonic,
    walletDataDir: paths.walletDataDir,
    compactDbPath: paths.compactDbPath,
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
