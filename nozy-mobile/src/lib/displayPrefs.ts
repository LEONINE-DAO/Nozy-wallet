import AsyncStorage from "@react-native-async-storage/async-storage";

export const HIDE_BALANCES_KEY = "nozy.display.hideBalances";

export async function loadHideBalances(): Promise<boolean> {
  const v = await AsyncStorage.getItem(HIDE_BALANCES_KEY);
  return v === "true";
}

export async function saveHideBalances(hide: boolean): Promise<void> {
  await AsyncStorage.setItem(HIDE_BALANCES_KEY, hide ? "true" : "false");
}
