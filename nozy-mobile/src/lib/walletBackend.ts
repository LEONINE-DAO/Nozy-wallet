import { isNozyWalletNativeAvailable } from "nozy-wallet";

export type WalletBackendMode = "companion" | "on_device";

/** True when a dev client ships `libnozy_ffi` / UniFFI bindings. */
export function isOnDeviceBackendAvailable(): boolean {
  try {
    return isNozyWalletNativeAvailable();
  } catch {
    return false;
  }
}
