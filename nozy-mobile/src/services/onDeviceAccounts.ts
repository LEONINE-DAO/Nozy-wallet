import AsyncStorage from "@react-native-async-storage/async-storage";

export const ONDEVICE_ACCOUNTS_KEY = "nozy.ondevice.accounts";
export const ONDEVICE_ACTIVE_ACCOUNT_KEY = "nozy.ondevice.activeAccount";

export type OnDeviceAccount = {
  index: number;
  name: string;
};

type Store = {
  accounts: OnDeviceAccount[];
  activeIndex: number;
};

const DEFAULT: Store = {
  accounts: [{ index: 0, name: "Personal" }],
  activeIndex: 0,
};

async function readStore(): Promise<Store> {
  const [raw, activeRaw] = await Promise.all([
    AsyncStorage.getItem(ONDEVICE_ACCOUNTS_KEY),
    AsyncStorage.getItem(ONDEVICE_ACTIVE_ACCOUNT_KEY),
  ]);
  let accounts: OnDeviceAccount[] = DEFAULT.accounts;
  if (raw) {
    try {
      const parsed = JSON.parse(raw) as OnDeviceAccount[];
      if (Array.isArray(parsed) && parsed.length > 0) {
        accounts = parsed
          .filter((a) => a && Number.isInteger(a.index) && a.index >= 0)
          .map((a) => ({
            index: a.index,
            name: (a.name || `Account ${a.index + 1}`).trim(),
          }));
      }
    } catch {
      /* keep default */
    }
  }
  if (!accounts.some((a) => a.index === 0)) {
    accounts = [{ index: 0, name: "Personal" }, ...accounts];
  }
  accounts.sort((a, b) => a.index - b.index);
  let activeIndex = 0;
  if (activeRaw != null && activeRaw !== "") {
    const n = Number(activeRaw);
    if (Number.isInteger(n) && accounts.some((a) => a.index === n)) {
      activeIndex = n;
    }
  }
  return { accounts, activeIndex };
}

async function writeStore(store: Store): Promise<void> {
  await AsyncStorage.multiSet([
    [ONDEVICE_ACCOUNTS_KEY, JSON.stringify(store.accounts)],
    [ONDEVICE_ACTIVE_ACCOUNT_KEY, String(store.activeIndex)],
  ]);
}

export async function ensureOnDeviceAccounts(): Promise<Store> {
  const store = await readStore();
  await writeStore(store);
  return store;
}

export async function listOnDeviceAccounts(): Promise<OnDeviceAccount[]> {
  return (await ensureOnDeviceAccounts()).accounts;
}

export async function getActiveOnDeviceAccount(): Promise<OnDeviceAccount> {
  const store = await ensureOnDeviceAccounts();
  return (
    store.accounts.find((a) => a.index === store.activeIndex) ??
    store.accounts[0]
  );
}

export async function getActiveOnDeviceAccountIndex(): Promise<number> {
  return (await getActiveOnDeviceAccount()).index;
}

export async function setActiveOnDeviceAccount(index: number): Promise<void> {
  const store = await ensureOnDeviceAccounts();
  if (!store.accounts.some((a) => a.index === index)) {
    throw new Error("That account does not exist");
  }
  store.activeIndex = index;
  await writeStore(store);
}

export async function renameOnDeviceAccount(
  index: number,
  name: string,
): Promise<void> {
  const trimmed = name.trim();
  if (!trimmed) throw new Error("Name is required");
  const store = await ensureOnDeviceAccounts();
  const row = store.accounts.find((a) => a.index === index);
  if (!row) throw new Error("That account does not exist");
  row.name = trimmed;
  await writeStore(store);
}

export async function addOnDeviceAccount(name?: string): Promise<OnDeviceAccount> {
  const store = await ensureOnDeviceAccounts();
  const nextIndex = store.accounts.reduce((m, a) => Math.max(m, a.index), -1) + 1;
  if (nextIndex > 20) {
    throw new Error("Maximum of 21 accounts on this seed");
  }
  const account: OnDeviceAccount = {
    index: nextIndex,
    name: (name?.trim() || `Account ${nextIndex + 1}`).trim(),
  };
  store.accounts.push(account);
  store.activeIndex = account.index;
  await writeStore(store);
  return account;
}
