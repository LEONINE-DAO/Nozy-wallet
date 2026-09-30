import { useState } from "react";
import { extensionApi } from "../lib/extensionApi";
import { splitMnemonicWords } from "../lib/seedBackup";
import { Button, Callout, Card, CopyButton, Hint, Input, SectionTitle } from "./ui";

/**
 * Settings: re-view / copy the recovery phrase after unlock (password step-up).
 */
export function SeedRevealCard() {
  const [password, setPassword] = useState("");
  const [mnemonic, setMnemonic] = useState<string | null>(null);
  const [revealed, setRevealed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reveal = async () => {
    setBusy(true);
    setError(null);
    const timeoutMs = 45_000;
    let timedOut = false;
    const timer = window.setTimeout(() => {
      timedOut = true;
      setBusy(false);
      setError(
        "Reveal is taking too long. Unlock again (same password), then retry Reveal — or reload the extension."
      );
    }, timeoutMs);
    try {
      const res = await extensionApi.walletRevealMnemonic(password);
      if (timedOut) return;
      const phrase = String(res?.mnemonic ?? "").trim();
      if (!phrase) {
        throw new Error("No recovery phrase returned. Try unlocking again, then reveal.");
      }
      setMnemonic(phrase);
      setRevealed(true);
      setPassword("");
    } catch (e) {
      if (timedOut) return;
      setError((e as Error).message);
      setMnemonic(null);
      setRevealed(false);
    } finally {
      window.clearTimeout(timer);
      if (!timedOut) setBusy(false);
    }
  };

  const hide = () => {
    setMnemonic(null);
    setRevealed(false);
    setPassword("");
    setError(null);
  };

  const words = mnemonic ? splitMnemonicWords(mnemonic) : [];

  return (
    <Card className="space-y-2.5">
      <SectionTitle>Recovery phrase</SectionTitle>
      <Hint>
        View or copy your 24-word backup anytime. Enter the same password you used to unlock.
      </Hint>

      {!revealed || !mnemonic ? (
        <>
          <Input
            label="Password"
            type="password"
            placeholder="Wallet password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && password && !busy) void reveal();
            }}
          />
          {busy && (
            <Hint>Checking password… usually instant after unlock.</Hint>
          )}
          {error && <Callout tone="danger">{error}</Callout>}
          <Button
            variant="secondary"
            fullWidth
            disabled={!password || busy}
            onClick={() => void reveal()}
          >
            {busy ? "Checking…" : "Reveal recovery phrase"}
          </Button>
        </>
      ) : (
        <>
          <Callout tone="warn">
            Keep this private. Close this panel when you’re done.
          </Callout>
          <ol
            className="grid grid-cols-2 gap-1.5 rounded-xl p-2.5 text-left"
            style={{ background: "var(--nw-surface-alt)" }}
          >
            {words.map((w, i) => (
              <li
                key={`${i}-${w}`}
                className="flex items-baseline gap-1.5 rounded-lg px-2 py-1 text-xs"
                style={{ background: "var(--nw-surface)" }}
              >
                <span className="nw-mono tabular-nums" style={{ color: "var(--nw-muted)" }}>
                  {i + 1}.
                </span>
                <span className="font-semibold">{w}</span>
              </li>
            ))}
          </ol>
          <div className="flex flex-wrap gap-2">
            <CopyButton value={mnemonic} label="Copy phrase" />
            <Button size="sm" variant="ghost" onClick={hide}>
              Hide phrase
            </Button>
          </div>
        </>
      )}
    </Card>
  );
}
