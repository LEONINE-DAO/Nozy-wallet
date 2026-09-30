import { useMemo, useState } from "react";
import {
  answersMatchQuiz,
  buildSeedVerifyQuiz,
  splitMnemonicWords,
  type SeedQuizItem
} from "../lib/seedBackup";
import { Button, Callout, Card, Hint, SectionTitle } from "./ui";

type SeedBackupPanelProps = {
  mnemonic: string;
  onConfirmed: () => void;
};

/**
 * Standard create-time backup: view/copy the 24 words, then confirm by
 * picking the correct word for a few random positions.
 */
export function SeedBackupPanel({ mnemonic, onConfirmed }: SeedBackupPanelProps) {
  const words = useMemo(() => splitMnemonicWords(mnemonic), [mnemonic]);
  const [revealed, setRevealed] = useState(false);
  const [copied, setCopied] = useState(false);
  const [step, setStep] = useState<"backup" | "verify">("backup");
  const [quiz, setQuiz] = useState<SeedQuizItem[] | null>(null);
  const [answers, setAnswers] = useState<Record<number, string>>({});
  const [error, setError] = useState<string | null>(null);

  const copyPhrase = async () => {
    try {
      await navigator.clipboard.writeText(mnemonic);
      setCopied(true);
      setError(null);
    } catch {
      setError("Could not copy — reveal the phrase and write it down instead.");
    }
  };

  const startVerify = () => {
    setError(null);
    try {
      const next = buildSeedVerifyQuiz(mnemonic, 3, 3);
      setQuiz(next);
      setAnswers({});
      setStep("verify");
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const canStartVerify = revealed || copied;

  const confirmVerify = () => {
    if (!quiz) return;
    if (!answersMatchQuiz(quiz, answers)) {
      setError("One or more words don’t match. Check your written copy and try again.");
      return;
    }
    setError(null);
    onConfirmed();
  };

  if (step === "verify" && quiz) {
    return (
      <Card className="space-y-3">
        <SectionTitle>Confirm you saved it</SectionTitle>
        <Hint>
          Select the correct word for each position below. This proves you wrote the phrase down.
        </Hint>
        {quiz.map((q) => (
          <div key={q.position} className="space-y-1.5">
            <p className="text-xs font-semibold" style={{ color: "var(--nw-muted)" }}>
              Word #{q.position}
            </p>
            <div className="flex flex-wrap gap-1.5">
              {q.choices.map((choice) => {
                const selected = answers[q.position] === choice;
                return (
                  <button
                    key={`${q.position}-${choice}`}
                    type="button"
                    className="rounded-lg px-2.5 py-1.5 text-xs font-semibold transition-colors"
                    style={
                      selected
                        ? { background: "var(--nw-platinum)", color: "#18181b" }
                        : {
                            background: "var(--nw-surface-alt)",
                            color: "var(--nw-fg)",
                            border: "1px solid var(--nw-border)"
                          }
                    }
                    onClick={() =>
                      setAnswers((prev) => ({ ...prev, [q.position]: choice }))
                    }
                  >
                    {choice}
                  </button>
                );
              })}
            </div>
          </div>
        ))}
        {error && <Callout tone="danger">{error}</Callout>}
        <div className="flex flex-wrap gap-2">
          <Button
            variant="primary"
            fullWidth
            disabled={quiz.some((q) => !answers[q.position])}
            onClick={confirmVerify}
          >
            Continue
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => {
              setStep("backup");
              setError(null);
            }}
          >
            Back to phrase
          </Button>
        </div>
      </Card>
    );
  }

  return (
    <Card className="space-y-3">
      <SectionTitle>Save your recovery phrase</SectionTitle>
      <Callout tone="warn">
        Write these 24 words offline. Anyone with this phrase can spend your funds. Nozy cannot
        recover it for you.
      </Callout>

      {!revealed ? (
        <button
          type="button"
          className="w-full rounded-xl px-3 py-8 text-sm font-semibold"
          style={{
            background: "var(--nw-surface-alt)",
            border: "1px dashed var(--nw-border)",
            color: "var(--nw-muted)"
          }}
          onClick={() => setRevealed(true)}
        >
          Tap to reveal recovery phrase
        </button>
      ) : (
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
      )}

      <div className="flex flex-wrap gap-2">
        {revealed ? (
          <Button size="sm" variant="ghost" onClick={() => setRevealed(false)}>
            Hide phrase
          </Button>
        ) : null}
        <Button
          size="sm"
          variant="secondary"
          icon={copied ? "check" : "copy"}
          fullWidth={!revealed}
          onClick={() => void copyPhrase()}
        >
          {copied ? "Copied" : "Copy phrase"}
        </Button>
      </div>

      {error && <Callout tone="danger">{error}</Callout>}

      <Button variant="primary" fullWidth disabled={!canStartVerify} onClick={startVerify}>
        I’ve saved it — verify
      </Button>
      <Hint className="text-center">
        Reveal or copy the phrase, then confirm a few word numbers before continuing.
      </Hint>
    </Card>
  );
}
