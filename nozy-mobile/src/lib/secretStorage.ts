import AsyncStorage from "@react-native-async-storage/async-storage";
import * as SecureStore from "expo-secure-store";

let secureQueue: Promise<unknown> = Promise.resolve();

function enqueueSecure<T>(task: () => Promise<T>): Promise<T> {
  const run = secureQueue.then(task, task);
  secureQueue = run.then(
    () => undefined,
    () => undefined,
  );
  return run;
}

const secureInflight = new Map<string, Promise<string | null>>();

/** Read a value that was copied into secure storage, without removing the app copy. */
function readSecure(key: string): Promise<string | null> {
  const existing = secureInflight.get(key);
  if (existing) return existing;
  const task = enqueueSecure(async () => {
    try {
      const available = await SecureStore.isAvailableAsync();
      if (!available) return null;
      return await SecureStore.getItemAsync(key);
    } catch {
      return null;
    }
  });
  secureInflight.set(key, task);
  void task.finally(() => {
    if (secureInflight.get(key) === task) secureInflight.delete(key);
  });
  return task;
}

/**
 * The wallet seed and password stay in app storage, which is what unlock used
 * before. If a previous run moved them into secure storage, copy them back.
 */
export async function getSecret(key: string): Promise<string | null> {
  const stored = await AsyncStorage.getItem(key);
  if (stored != null && stored !== "") return stored;

  const recovered = await readSecure(key);
  if (recovered != null && recovered !== "") {
    await AsyncStorage.setItem(key, recovered);
    return recovered;
  }
  return null;
}

export async function setSecret(key: string, value: string): Promise<void> {
  await AsyncStorage.setItem(key, value);
}

export async function deleteSecret(key: string): Promise<void> {
  await AsyncStorage.removeItem(key);
}
