/** BIP-39 recovery phrase backup helpers (create confirm + settings reveal). */

export function splitMnemonicWords(mnemonic: string): string[] {
  return mnemonic
    .trim()
    .split(/\s+/)
    .filter(Boolean);
}

export type SeedQuizItem = {
  /** 1-based word position shown to the user */
  position: number;
  /** 0-based index into the mnemonic word list */
  index: number;
  correct: string;
  /** Shuffled choices including the correct word */
  choices: string[];
};

function shuffleInPlace<T>(arr: T[], rand: () => number): T[] {
  for (let i = arr.length - 1; i > 0; i -= 1) {
    const j = Math.floor(rand() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
  return arr;
}

/**
 * Build a short confirmation quiz: pick `count` random word positions and
 * offer multiple-choice answers (correct + distractors from the same phrase).
 */
export function buildSeedVerifyQuiz(
  mnemonic: string,
  count = 3,
  choicesPerQuestion = 3,
  rand: () => number = Math.random
): SeedQuizItem[] {
  const words = splitMnemonicWords(mnemonic);
  if (words.length < 12) {
    throw new Error("Recovery phrase looks incomplete.");
  }
  const n = Math.min(count, words.length);
  const indices = shuffleInPlace(
    Array.from({ length: words.length }, (_, i) => i),
    rand
  ).slice(0, n);

  return indices.map((index) => {
    const correct = words[index]!;
    const distractorPool = words.filter((w, i) => i !== index && w !== correct);
    const uniqueDistractors = [...new Set(distractorPool)];
    shuffleInPlace(uniqueDistractors, rand);
    const distractors = uniqueDistractors.slice(0, Math.max(0, choicesPerQuestion - 1));
    // If the phrase has too few unique distractors, pad with other BIP39-looking fillers
    // from elsewhere in the phrase (already unique) or the correct word once only.
    while (distractors.length < choicesPerQuestion - 1 && uniqueDistractors.length > distractors.length) {
      distractors.push(uniqueDistractors[distractors.length]!);
    }
    const choices = shuffleInPlace([correct, ...distractors], rand);
    return {
      position: index + 1,
      index,
      correct,
      choices
    };
  });
}

export function answersMatchQuiz(
  quiz: SeedQuizItem[],
  answers: Record<number, string>
): boolean {
  return quiz.every((q) => (answers[q.position] ?? "").trim().toLowerCase() === q.correct.toLowerCase());
}
