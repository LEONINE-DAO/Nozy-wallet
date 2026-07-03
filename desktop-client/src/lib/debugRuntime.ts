import { isTauri as isTauriApi } from "@tauri-apps/api/core";

export type RuntimeContext = ReturnType<typeof captureRuntimeContext>;

export function captureRuntimeContext() {
  if (typeof window === "undefined") {
    return {
      href: "",
      protocol: "",
      host: "",
      hasTauriInternals: false,
      hasTauriGlobal: false,
      hasIsTauriFlag: false,
      isTauriApi: false,
      userAgent: "",
      isSecureContext: false,
    };
  }

  const w = window as Window & {
    __TAURI_INTERNALS__?: unknown;
    __TAURI__?: unknown;
    isTauri?: boolean;
  };

  return {
    href: window.location.href,
    protocol: window.location.protocol,
    host: window.location.host,
    hasTauriInternals: typeof w.__TAURI_INTERNALS__ !== "undefined",
    hasTauriGlobal: typeof w.__TAURI__ !== "undefined",
    hasIsTauriFlag: !!w.isTauri,
    isTauriApi: isTauriApi(),
    userAgent: navigator.userAgent,
    isSecureContext: window.isSecureContext,
  };
}

export function hasTauriRuntime() {
  return captureRuntimeContext().isTauriApi;
}

/** User-facing hint when IPC is unavailable (evidence: Cursor preview uses Electron UA without __TAURI_INTERNALS__). */
export function describeNonTauriHost(ctx: RuntimeContext = captureRuntimeContext()): string {
  const ua = ctx.userAgent;
  if (ua.includes("Brave") || (ua.includes("Chrome/") && !ua.includes("Edg/") && !ctx.isTauriApi)) {
    return "Brave/Chrome cannot run the wallet backend. Close this tab and use the NozyWallet desktop window from the Windows taskbar.";
  }
  if (ua.includes("Cursor/")) {
    return "You opened this in Cursor's built-in browser/preview. Close it and use the NozyWallet desktop window from the Windows taskbar (nozywallet-desktop.exe).";
  }
  if (ua.includes("Electron/") && !ctx.isTauriApi) {
    return "This preview panel has no wallet backend. Use the NozyWallet desktop window from the taskbar instead.";
  }
  if (ctx.host.includes("localhost:5173")) {
    return "You opened the Vite dev URL in a browser tab. Close it and use the NozyWallet desktop window instead.";
  }
  return "Launch the NozyWallet desktop app: from desktop-client run npm run tauri dev.";
}
