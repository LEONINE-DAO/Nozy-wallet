import { api } from "./api";
import { onDeviceCompactSync } from "./onDeviceWallet";

/** Matches the Sync setting: check for new shielded notes while unlocked. */
export const INCOMING_SYNC_MS = 20_000;

export type WalletSyncSnapshot = {
  chainTip: number;
  unspentZec: number;
  orchardZec?: number;
  ironwoodZec?: number;
};

type WalletSyncEvent =
  | { type: "start" }
  | { type: "done"; snapshot: WalletSyncSnapshot }
  | { type: "error"; message: string };

type Listener = (event: WalletSyncEvent) => void;

const listeners = new Set<Listener>();
let inFlight: Promise<void> | null = null;

export function subscribeWalletSync(listener: Listener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function emit(event: WalletSyncEvent) {
  for (const listener of listeners) listener(event);
}

/**
 * One compact sync at a time. Callers that start while a sync is already
 * running share that run, so a 20s timer cannot stack on the native queue.
 */
export function requestWalletSync(opts: {
  onDevice: boolean;
  password?: string;
}): Promise<void> {
  if (inFlight) return inFlight;
  inFlight = runWalletSync(opts).finally(() => {
    inFlight = null;
  });
  return inFlight;
}

async function runWalletSync(opts: {
  onDevice: boolean;
  password?: string;
}): Promise<void> {
  emit({ type: "start" });
  try {
    if (opts.onDevice) {
      const result = await onDeviceCompactSync();
      emit({
        type: "done",
        snapshot: {
          chainTip: result.chainTip,
          unspentZec: result.unspentZec,
          orchardZec: result.orchardZec,
          ironwoodZec: result.ironwoodZec,
        },
      });
      return;
    }
    const result = await api.syncWallet(opts.password || undefined);
    emit({
      type: "done",
      snapshot: {
        chainTip: result.chain_tip,
        unspentZec: result.balance_zec,
      },
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : "Sync failed";
    emit({ type: "error", message });
    throw error;
  }
}
