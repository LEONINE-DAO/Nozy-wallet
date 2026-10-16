/** ZIP-32 accounts under one extension seed. Indices 0 through 20. */

export const ACCOUNT_INDEX_MAX = 20;

export function emptyAccountStore() {
  return {
    accounts: [{ index: 0, name: "Personal" }],
    activeIndex: 0
  };
}

function accountName(index, name) {
  const trimmed = String(name || "").trim();
  return trimmed || `Account ${index + 1}`;
}

export function normalizeAccountStore(raw) {
  const source = raw && typeof raw === "object" ? raw : {};
  const accounts = [];
  if (Array.isArray(source.accounts)) {
    for (const row of source.accounts) {
      const index = Number(row?.index);
      if (!Number.isInteger(index) || index < 0 || index > ACCOUNT_INDEX_MAX) continue;
      if (accounts.some((account) => account.index === index)) continue;
      accounts.push({ index, name: accountName(index, row?.name) });
    }
  }
  if (!accounts.some((account) => account.index === 0)) {
    accounts.unshift({ index: 0, name: "Personal" });
  }
  accounts.sort((a, b) => a.index - b.index);
  let activeIndex = Number(source.activeIndex);
  if (!Number.isInteger(activeIndex) || !accounts.some((account) => account.index === activeIndex)) {
    activeIndex = 0;
  }
  return { accounts, activeIndex };
}

export function addAccount(store, name) {
  const current = normalizeAccountStore(store);
  const nextIndex = current.accounts.reduce((max, account) => Math.max(max, account.index), -1) + 1;
  if (nextIndex > ACCOUNT_INDEX_MAX) {
    throw new Error("Maximum of 21 accounts on this seed");
  }
  const account = { index: nextIndex, name: accountName(nextIndex, name) };
  return {
    accounts: [...current.accounts, account],
    activeIndex: account.index
  };
}

export function renameAccount(store, index, name) {
  const current = normalizeAccountStore(store);
  const target = Number(index);
  if (!current.accounts.some((account) => account.index === target)) {
    throw new Error("That account does not exist");
  }
  const trimmed = String(name || "").trim();
  if (!trimmed) throw new Error("Name is required");
  return {
    accounts: current.accounts.map((account) =>
      account.index === target ? { ...account, name: trimmed } : account
    ),
    activeIndex: current.activeIndex
  };
}

export function setActiveAccount(store, index) {
  const current = normalizeAccountStore(store);
  const target = Number(index);
  if (!current.accounts.some((account) => account.index === target)) {
    throw new Error("That account does not exist");
  }
  return { accounts: current.accounts, activeIndex: target };
}
