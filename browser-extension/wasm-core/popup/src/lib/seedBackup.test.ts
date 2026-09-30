import { describe, expect, it } from "vitest";
import {
  answersMatchQuiz,
  buildSeedVerifyQuiz,
  splitMnemonicWords
} from "./seedBackup";

const PHRASE =
  "legal winner thank year wave sausage worth useful legal winner thank yellow";

describe("seedBackup", () => {
  it("splits mnemonic words", () => {
    expect(splitMnemonicWords(`  ${PHRASE}  `)).toHaveLength(12);
  });

  it("builds a quiz with numbered positions and correct among choices", () => {
    const words = splitMnemonicWords(PHRASE);
    let i = 0;
    const rand = () => {
      // Deterministic but shuffled enough for tests.
      i += 1;
      return (i % 10) / 10;
    };
    const quiz = buildSeedVerifyQuiz(PHRASE, 3, 3, rand);
    expect(quiz).toHaveLength(3);
    for (const q of quiz) {
      expect(q.position).toBe(q.index + 1);
      expect(words[q.index]).toBe(q.correct);
      expect(q.choices).toContain(q.correct);
      expect(q.choices.length).toBeGreaterThanOrEqual(2);
    }
  });

  it("answersMatchQuiz requires exact words", () => {
    const quiz = buildSeedVerifyQuiz(PHRASE, 2, 3, () => 0.2);
    const good: Record<number, string> = {};
    for (const q of quiz) good[q.position] = q.correct;
    expect(answersMatchQuiz(quiz, good)).toBe(true);
    expect(answersMatchQuiz(quiz, { [quiz[0]!.position]: "wrong" })).toBe(false);
  });
});
