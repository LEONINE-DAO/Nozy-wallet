import type { PopupView } from "../store/uiStore";

const PAGE_FILE = "wasm-core/popup/dist/index.html";

const PAGE_VIEWS: PopupView[] = [
  "dashboard",
  "send",
  "receive",
  "settings",
  "vote",
  "crosslink",
  "companion",
  "browser"
];

export function isFullPage(): boolean {
  try {
    return new URLSearchParams(window.location.search).get("page") === "1";
  } catch {
    return false;
  }
}

export function viewFromUrl(): PopupView | null {
  try {
    const v = new URLSearchParams(window.location.search).get("view");
    if (v && PAGE_VIEWS.includes(v as PopupView)) return v as PopupView;
  } catch {
    /* ignore */
  }
  return null;
}

export function finalizerFromUrl(): string | null {
  try {
    const hex = new URLSearchParams(window.location.search).get("finalizer");
    return hex && hex.trim() ? hex.trim() : null;
  } catch {
    return null;
  }
}

export function applyWalletPageClass(): void {
  if (isFullPage()) {
    document.documentElement.classList.add("nw-page");
    document.title = "NozyWallet";
  }
}

/** Open the same wallet UI in a browser tab (Keplr / Brave full-page, not the 400×600 popup). */
export async function openWalletPage(opts?: {
  view?: PopupView;
  finalizer?: string;
}): Promise<void> {
  if (typeof chrome === "undefined" || !chrome.runtime?.getURL || !chrome.tabs?.create) {
    return;
  }
  if (isFullPage()) return;
  const url = new URL(chrome.runtime.getURL(PAGE_FILE));
  url.searchParams.set("page", "1");
  if (opts?.view) url.searchParams.set("view", opts.view);
  if (opts?.finalizer) url.searchParams.set("finalizer", opts.finalizer);
  await chrome.tabs.create({ url: url.toString() });
  window.close();
}
