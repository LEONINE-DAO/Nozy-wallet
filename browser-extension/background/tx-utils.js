export function selectNotesForSpend(notes, requiredValue) {
  const usable = notes
    .filter((n) => Number.isFinite(n.value) && n.value > 0)
    .sort((a, b) => a.value - b.value || a.height - b.height);

  if (usable.length === 0) return [];

  // Prefer a single smallest-sufficient note first.
  const single = usable.find((n) => n.value >= requiredValue);
  if (single) return [single];

  // Otherwise, accumulate larger notes first to minimize input count.
  const desc = [...usable].sort((a, b) => b.value - a.value || a.height - b.height);
  const selected = [];
  let running = 0;
  for (const note of desc) {
    selected.push(note);
    running += note.value;
    if (running >= requiredValue) break;
  }
  return running >= requiredValue ? selected : [];
}

export async function rpcFallbackWithRequester(requester, attempts) {
  let lastErr;
  for (const at of attempts) {
    try {
      return await requester(at);
    } catch (err) {
      lastErr = err;
    }
  }
  throw lastErr || new Error("All RPC fallbacks failed");
}

