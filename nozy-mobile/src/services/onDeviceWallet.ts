import AsyncStorage from "@react-native-async-storage/async-storage";
import {
  generateMnemonic,
  orchardUnifiedAddress,
  orchardScan,
  orchardStatus,
  lwdGetInfo,
  lwdSyncCompactToTip,
  saplingScan,
  saplingStatus,
  validateMnemonic,
  walletPaths,
  type UnifiedAddressNative,
} from "nozy-wallet";
import {
  ONDEVICE_COMPACT_DB_KEY,
  ONDEVICE_DATA_DIR_KEY,
  ONDEVICE_LWD_URL_KEY,
  ONDEVICE_MNEMONIC_KEY,
  ONDEVICE_PASSWORD_KEY,
  ONDEVICE_WALLET_KIND_KEY,
} from "../components/settings/OnDeviceWalletSettings";
import { defaultHostedLwdUrl } from "../lib/connectionPresets";
import {
  NU5_ORCHARD_MAINNET,
  parseRestoreHeight,
} from "../lib/zcashHeights";
import {
  loadOnDeviceBirthday,
  loadOnDevicePaths,
  persistOnDeviceBirthday,
} from "./onDeviceSapling";
import { ensureOnDeviceAccounts, getActiveOnDeviceAccountIndex, listOnDeviceAccounts } from "./onDeviceAccounts";

export type OnDeviceWalletMeta = {
  exists: boolean;
  hasPassword: boolean;
};

async function applyDefaultPaths(): Promise<{
  walletDataDir: string;
  compactDbPath: string;
}> {
  const nativePaths = walletPaths();
  const [storedDir, storedDb, storedLwd] = await Promise.all([
    AsyncStorage.getItem(ONDEVICE_DATA_DIR_KEY),
    AsyncStorage.getItem(ONDEVICE_COMPACT_DB_KEY),
    AsyncStorage.getItem(ONDEVICE_LWD_URL_KEY),
  ]);
  const walletDataDir = storedDir?.trim() || nativePaths.walletDataDir;
  const compactDbPath = storedDb?.trim() || nativePaths.compactDbPath;
  await AsyncStorage.multiSet([
    [ONDEVICE_DATA_DIR_KEY, walletDataDir],
    [ONDEVICE_COMPACT_DB_KEY, compactDbPath],
    [
      ONDEVICE_LWD_URL_KEY,
      storedLwd?.trim() || defaultHostedLwdUrl(),
    ],
  ]);
  return { walletDataDir, compactDbPath };
}

export async function onDeviceWalletMeta(): Promise<OnDeviceWalletMeta> {
  const [mnemonic, password] = await Promise.all([
    AsyncStorage.getItem(ONDEVICE_MNEMONIC_KEY),
    AsyncStorage.getItem(ONDEVICE_PASSWORD_KEY),
  ]);
  return {
    exists: Boolean(mnemonic?.trim()),
    hasPassword: Boolean(password),
  };
}

async function birthdayForNewWallet(lwdUrl: string): Promise<number> {
  try {
    const info = await lwdGetInfo(lwdUrl);
    const h = Number(info.block_height) || Number(info.estimated_height) || 0;
    if (h > 0) return Math.floor(h);
  } catch {
    // LWD unreachable at create — still do not scan from height 1.
  }
  return NU5_ORCHARD_MAINNET;
}

export async function persistOnDeviceWallet(params: {
  mnemonic: string;
  password: string;
  birthday: number;
  kind: "create" | "restore";
}): Promise<void> {
  await validateMnemonic(params.mnemonic.trim());
  await applyDefaultPaths();
  await AsyncStorage.setItem(ONDEVICE_MNEMONIC_KEY, params.mnemonic.trim());
  await persistOnDeviceBirthday(params.birthday);
  await AsyncStorage.setItem(ONDEVICE_WALLET_KIND_KEY, params.kind);
  if (params.password) {
    await AsyncStorage.setItem(ONDEVICE_PASSWORD_KEY, params.password);
  } else {
    await AsyncStorage.removeItem(ONDEVICE_PASSWORD_KEY);
  }
  await ensureOnDeviceAccounts();
}

export async function createOnDeviceWallet(password: string): Promise<string> {
  const mnemonic = await generateMnemonic();
  const lwdStored = await AsyncStorage.getItem(ONDEVICE_LWD_URL_KEY);
  const lwd = lwdStored?.trim() || defaultHostedLwdUrl();
  const birthday = await birthdayForNewWallet(lwd);
  await persistOnDeviceWallet({ mnemonic, password, birthday, kind: "create" });
  return mnemonic;
}

export async function restoreOnDeviceWallet(
  mnemonic: string,
  password: string,
  restoreHeight?: string | number | null,
): Promise<void> {
  await persistOnDeviceWallet({
    mnemonic: mnemonic.trim(),
    password,
    birthday: parseRestoreHeight(restoreHeight),
    kind: "restore",
  });
}

export async function unlockOnDeviceWallet(password: string): Promise<void> {
  const [mnemonic, stored] = await Promise.all([
    AsyncStorage.getItem(ONDEVICE_MNEMONIC_KEY),
    AsyncStorage.getItem(ONDEVICE_PASSWORD_KEY),
  ]);
  if (!mnemonic?.trim()) {
    throw new Error("No wallet on this phone. Create or restore one first.");
  }
  if (stored && stored !== password) {
    throw new Error("Wrong password.");
  }
  if (!stored && password.trim()) {
    throw new Error("This wallet has no password. Leave the field empty.");
  }
  await validateMnemonic(mnemonic.trim());
}

export async function onDeviceSpendableZec(): Promise<number> {
  const snap = await onDeviceShieldedSnapshot();
  return snap.unspentZec;
}

export type OnDeviceShieldedSnapshot = {
  unspentZec: number;
  orchardZec: number;
  ironwoodZec: number;
  notes: {
    txid: string;
    block_height: number;
    value_zec: number;
    spent: boolean;
    pool: string;
  }[];
};

export async function onDeviceShieldedSnapshot(): Promise<OnDeviceShieldedSnapshot> {
  const empty: OnDeviceShieldedSnapshot = {
    unspentZec: 0,
    orchardZec: 0,
    ironwoodZec: 0,
    notes: [],
  };
  const paths = await loadOnDevicePaths();
  if (!paths) return empty;
  try {
    const status = await orchardStatus(paths.walletDataDir);
    let orchardZec = 0;
    let ironwoodZec = 0;
    for (const note of status.notes) {
      if (note.spent) continue;
      if (note.pool === "ironwood") ironwoodZec += note.value_zec;
      else orchardZec += note.value_zec;
    }
    return {
      unspentZec: status.unspent_zec,
      orchardZec,
      ironwoodZec,
      notes: status.notes,
    };
  } catch {
    try {
      const status = await saplingStatus(paths.walletDataDir);
      return { ...empty, unspentZec: Number(status.unspent_zec) || 0 };
    } catch {
      return empty;
    }
  }
}

export async function onDeviceReceiveAddress(): Promise<UnifiedAddressNative> {
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error("No on-device wallet. Create or restore first.");
  }
  const account = await getActiveOnDeviceAccountIndex();
  return orchardUnifiedAddress(paths.mnemonic, account);
}

export async function onDeviceCompactSync(): Promise<{
  chainTip: number;
  unspentZec: number;
  orchardZec: number;
  ironwoodZec: number;
}> {
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error("No on-device wallet. Create or restore first.");
  }
  const lwd = paths.lightwalletdUrl.trim() || defaultHostedLwdUrl();
  const startFloor = await loadOnDeviceBirthday();
  let chainTip: number;
  try {
    chainTip = await lwdSyncCompactToTip({
      lightwalletdUrl: lwd,
      compactDbPath: paths.compactDbPath,
      startFloor,
    });
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    const ahead = /end (\d+) < start (\d+)/i.exec(msg);
    if (ahead) {
      const rangeEnd = Number(ahead[1]);
      const rangeStart = Number(ahead[2]);
      if (
        Number.isFinite(rangeEnd) &&
        Number.isFinite(rangeStart) &&
        rangeStart > rangeEnd
      ) {
        chainTip = rangeEnd;
      } else {
        throw e;
      }
    } else if (/h2 protocol error|GetBlockRange/i.test(msg)) {
      throw new Error(
        `Lightwalletd dropped the compact-block stream (start ${startFloor}). Sync again from wallet birthday near tip — a long range fills the phone and the HTTP/2 stream dies.`,
      );
    } else {
      throw e;
    }
  }
  try {
    await saplingScan({
      mnemonic: paths.mnemonic,
      walletDataDir: paths.walletDataDir,
      compactDbPath: paths.compactDbPath,
      startFloor,
    });
  } catch {
    // Compact download succeeded; Sapling scan can catch up next tap.
  }
  try {
    const accounts = await listOnDeviceAccounts();
    const maxAccount = accounts.reduce((m, a) => Math.max(m, a.index), 0);
    await orchardScan({
      mnemonic: paths.mnemonic,
      walletDataDir: paths.walletDataDir,
      compactDbPath: paths.compactDbPath,
      startFloor,
      maxAccount,
      full: false,
    });
  } catch {
    // Compact download succeeded; Orchard/Ironwood scan can catch up next tap.
  }
  const snap = await onDeviceShieldedSnapshot();
  return {
    chainTip,
    unspentZec: snap.unspentZec,
    orchardZec: snap.orchardZec,
    ironwoodZec: snap.ironwoodZec,
  };
}
