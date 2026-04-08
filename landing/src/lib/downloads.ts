/**
 * Latest release page (always valid). Use for desktop/extension: those assets are
 * attached by follow-up workflows and are often missing from `/releases/latest/download/...`
 * until CI finishes — direct URLs would 404.
 */
export const REPO_RELEASES_LATEST =
  "https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest";

/** Direct asset URL when you know the file exists on the current latest release. */
export function releaseAsset(filename: string): string {
  return `${REPO_RELEASES_LATEST}/download/${encodeURIComponent(filename)}`;
}

/** Filenames must match CI-uploaded assets. CLI ships with every tag; desktop/extension may lag. */
export const DOWNLOAD_URLS = {
  /** Tauri / NSIS setup (Windows). */
  desktopWindowsNsis: releaseAsset("nozy-desktop-windows-x86_64-installer.exe"),
  /** Windows MSI installer. */
  desktopWindowsMsi: releaseAsset("nozy-desktop-windows-x86_64-installer.msi"),
  /** Chromium extension — load unpacked after unzip (Chrome, Edge, Brave). */
  extensionChromiumZip: releaseAsset("nozy-extension-chromium.zip"),
  /** Same bundle; Firefox testing only. */
  extensionFirefoxZip: releaseAsset("nozy-extension-firefox.zip"),
  cliWindows: releaseAsset("nozy-windows.exe"),
  cliLinux: releaseAsset("nozy-linux"),
  cliMacIntel: releaseAsset("nozy-macos-intel"),
  cliMacArm: releaseAsset("nozy-macos-arm")
} as const;
