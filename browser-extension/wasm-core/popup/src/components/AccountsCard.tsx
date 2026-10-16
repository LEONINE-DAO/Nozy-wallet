import { useCallback, useEffect, useState } from "react";
import { extensionApi, type WalletAccount } from "../lib/extensionApi";
import { Button, Callout, Card, Hint, Input, SectionTitle } from "./ui";

export function AccountsCard({
  unlocked,
  onChanged
}: {
  unlocked: boolean;
  onChanged?: () => void;
}) {
  const [accounts, setAccounts] = useState<WalletAccount[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [name, setName] = useState("");
  const [renameIndex, setRenameIndex] = useState<number | null>(null);
  const [renameDraft, setRenameDraft] = useState("");
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    const listed = await extensionApi.walletListAccounts();
    setAccounts(listed.accounts);
    setActiveIndex(listed.activeIndex);
  }, []);

  useEffect(() => {
    void load().catch((e: unknown) => {
      setError(e instanceof Error ? e.message : "Could not load accounts");
    });
  }, [load, unlocked]);

  async function createAccount() {
    setError("");
    setStatus("");
    if (!unlocked) {
      setError("Unlock the wallet before creating an account.");
      return;
    }
    setBusy(true);
    try {
      const created = await extensionApi.walletAddAccount(name);
      setName("");
      setAccounts(created.accounts);
      setActiveIndex(created.activeIndex);
      setStatus(`${created.activeAccount.name} is ready. Receive uses this account now.`);
      onChanged?.();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not create account");
    } finally {
      setBusy(false);
    }
  }

  async function switchAccount(index: number) {
    setError("");
    setStatus("");
    setBusy(true);
    try {
      const next = await extensionApi.walletSetActiveAccount(index);
      setAccounts(next.accounts);
      setActiveIndex(next.activeIndex);
      setStatus(`Receiving as ${next.activeAccount.name}.`);
      onChanged?.();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not switch account");
    } finally {
      setBusy(false);
    }
  }

  async function saveRename() {
    if (renameIndex == null) return;
    setError("");
    setStatus("");
    setBusy(true);
    try {
      const next = await extensionApi.walletRenameAccount(renameIndex, renameDraft);
      setAccounts(next.accounts);
      setActiveIndex(next.activeIndex);
      setRenameIndex(null);
      setRenameDraft("");
      setStatus("Account renamed.");
      onChanged?.();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not rename account");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card className="space-y-3">
      <SectionTitle>Accounts</SectionTitle>
      <Hint>
        Extra accounts stay on this recovery phrase. The balance is shared across them. Sync again
        after creating an account so older payments to it can be found.
      </Hint>
      <div className="flex items-end gap-2">
        <div className="min-w-0 flex-1">
          <Input
            label="New account"
            placeholder={`Account ${accounts.length + 1}`}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <Button size="sm" disabled={busy || !unlocked} onClick={() => void createAccount()}>
          Create
        </Button>
      </div>
      <div className="space-y-2">
        {accounts.map((account) => {
          const active = account.index === activeIndex;
          const renaming = renameIndex === account.index;
          return (
            <div
              key={account.index}
              className="rounded-xl px-3 py-2.5"
              style={{
                border: active ? "1px solid var(--nw-platinum)" : "1px solid var(--nw-border)",
                background: active ? "var(--nw-platinum-soft)" : "transparent"
              }}
            >
              {renaming ? (
                <div className="flex items-end gap-2">
                  <div className="min-w-0 flex-1">
                    <Input
                      label="Name"
                      value={renameDraft}
                      onChange={(e) => setRenameDraft(e.target.value)}
                    />
                  </div>
                  <Button size="sm" disabled={busy} onClick={() => void saveRename()}>
                    Save
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => {
                      setRenameIndex(null);
                      setRenameDraft("");
                    }}
                  >
                    Cancel
                  </Button>
                </div>
              ) : (
                <div className="flex items-center justify-between gap-2">
                  <div className="min-w-0">
                    <p className="truncate text-[13px] font-semibold">{account.name}</p>
                    <p className="text-[11px]" style={{ color: "var(--nw-muted)" }}>
                      Account {account.index}
                      {active ? " · Active" : ""}
                    </p>
                  </div>
                  <div className="flex shrink-0 gap-1">
                    {!active && (
                      <Button
                        size="sm"
                        variant="secondary"
                        disabled={busy || !unlocked}
                        onClick={() => void switchAccount(account.index)}
                      >
                        Use
                      </Button>
                    )}
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={busy}
                      onClick={() => {
                        setRenameIndex(account.index);
                        setRenameDraft(account.name);
                      }}
                    >
                      Rename
                    </Button>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
      {status && <Callout tone="success">{status}</Callout>}
      {error && <Callout tone="danger">{error}</Callout>}
    </Card>
  );
}
