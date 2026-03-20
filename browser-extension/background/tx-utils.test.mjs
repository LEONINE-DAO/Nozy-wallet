import test from "node:test";
import assert from "node:assert/strict";
import { rpcFallbackWithRequester, selectNotesForSpend } from "./tx-utils.js";

test("selectNotesForSpend prefers smallest single sufficient note", () => {
  const notes = [
    { value: 5000, height: 10 },
    { value: 11000, height: 11 },
    { value: 9000, height: 12 }
  ];
  const selected = selectNotesForSpend(notes, 8000);
  assert.equal(selected.length, 1);
  assert.equal(selected[0].value, 9000);
});

test("selectNotesForSpend accumulates multiple notes when needed", () => {
  const notes = [
    { value: 3000, height: 10 },
    { value: 4000, height: 12 },
    { value: 5000, height: 11 }
  ];
  const selected = selectNotesForSpend(notes, 9000);
  assert.ok(selected.length >= 2);
  const total = selected.reduce((acc, n) => acc + n.value, 0);
  assert.ok(total >= 9000);
});

test("selectNotesForSpend returns empty on insufficient funds", () => {
  const notes = [
    { value: 1000, height: 1 },
    { value: 2000, height: 2 }
  ];
  const selected = selectNotesForSpend(notes, 99999);
  assert.deepEqual(selected, []);
});

test("rpcFallbackWithRequester returns first successful attempt", async () => {
  let calls = 0;
  const result = await rpcFallbackWithRequester(async (attempt) => {
    calls += 1;
    if (attempt.method === "fail") throw new Error("nope");
    return "ok";
  }, [{ method: "fail" }, { method: "ok" }]);

  assert.equal(result, "ok");
  assert.equal(calls, 2);
});

