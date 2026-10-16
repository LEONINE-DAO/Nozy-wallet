import assert from "node:assert/strict";
import test from "node:test";
import {
  addAccount,
  emptyAccountStore,
  normalizeAccountStore,
  renameAccount,
  setActiveAccount
} from "./wallet-accounts.js";

test("starts with Personal on account 0", () => {
  assert.deepEqual(emptyAccountStore(), {
    accounts: [{ index: 0, name: "Personal" }],
    activeIndex: 0
  });
});

test("adds the next account and makes it active", () => {
  const next = addAccount(emptyAccountStore(), "Savings");
  assert.deepEqual(next.accounts, [
    { index: 0, name: "Personal" },
    { index: 1, name: "Savings" }
  ]);
  assert.equal(next.activeIndex, 1);
});

test("renames and switches without dropping account 0", () => {
  const created = addAccount(emptyAccountStore(), "");
  const renamed = renameAccount(created, 1, "Business");
  const back = setActiveAccount(renamed, 0);
  assert.equal(back.accounts[1].name, "Business");
  assert.equal(back.activeIndex, 0);
});

test("keeps account 0 when storage is empty or corrupt", () => {
  assert.deepEqual(normalizeAccountStore(null).accounts[0], { index: 0, name: "Personal" });
  assert.equal(
    normalizeAccountStore({ accounts: [{ index: 4, name: "Later" }] }).accounts[0].index,
    0
  );
});

test("stops at 21 accounts", () => {
  let store = emptyAccountStore();
  for (let i = 0; i < 20; i += 1) store = addAccount(store);
  assert.equal(store.accounts.length, 21);
  assert.throws(() => addAccount(store, "One more"), /21 accounts/);
});
