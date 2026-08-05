/**
 * Download URLs for GitHub Releases.
 * CLI + companion API assets ship on the latest (non-prerelease) CLI tag via /releases/latest.
 * Desktop ships on its own tag (Hot Lemon Pepper Sprinkles line) — pin the tag so links stay stable.
 */
export const REPO_RELEASES_LATEST =
  "https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest";

/** Current desktop release tag (food name: Hot Lemon Pepper Sprinkles). */
export const DESKTOP_TAG = "desktop-v1.0.0-beta.6";
export const DESKTOP_RELEASE =
  `https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/${DESKTOP_TAG}`;

/** @deprecated Use DESKTOP_TAG — kept for any stray imports. */
export const DESKTOP_BETA_TAG = DESKTOP_TAG;
/** @deprecated Use DESKTOP_RELEASE */
export const DESKTOP_BETA_RELEASE = DESKTOP_RELEASE;

/** Direct asset URL on the current latest (non-prerelease) release. */
export function releaseAsset(filename: string): string {
  return `${REPO_RELEASES_LATEST}/download/${encodeURIComponent(filename)}`;
}

export function desktopAsset(filename: string): string {
  return `https://github.com/LEONINE-DAO/Nozy-wallet/releases/download/${DESKTOP_TAG}/${encodeURIComponent(filename)}`;
}

/** @deprecated Use desktopAsset */
export function desktopBetaAsset(filename: string): string {
  return desktopAsset(filename);
}

/** Production CLI + companion API assets (attached on every CLI tag). */
export const DOWNLOAD_URLS = {
  cliWindows: releaseAsset("nozy-windows.exe"),
  cliLinux: releaseAsset("nozy-linux"),
  cliMacIntel: releaseAsset("nozy-macos-intel"),
  cliMacArm: releaseAsset("nozy-macos-arm"),
  apiWindows: releaseAsset("nozywallet-api-windows.exe"),
  apiLinux: releaseAsset("nozywallet-api-linux"),
  apiMacIntel: releaseAsset("nozywallet-api-macos-intel"),
  apiMacArm: releaseAsset("nozywallet-api-macos-arm"),
  hashes: releaseAsset("HASHES.txt"),
} as const;

/** Desktop installer / binaries (Hot Lemon Pepper Sprinkles — desktop-v1.0.0-beta.6). */
export const DESKTOP_DOWNLOAD_URLS = {
  windows: desktopAsset("nozy-desktop-windows-x86_64-installer.exe"),
  linux: desktopAsset("nozy-desktop-linux-x86_64.tar.gz"),
  macArm: desktopAsset("nozy-desktop-macos-aarch64.tar.gz"),
  releasePage: DESKTOP_RELEASE,
} as const;
