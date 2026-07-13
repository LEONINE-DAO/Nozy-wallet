/**
 * Download URLs for GitHub Releases.
 * CLI assets ship on the latest (non-prerelease) tag via /releases/latest.
 * Desktop beta is a prerelease — pin the tag so links stay stable.
 */
export const REPO_RELEASES_LATEST =
  "https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest";

export const DESKTOP_BETA_TAG = "desktop-v1.1.0-beta2-Hot-lemon";
export const DESKTOP_BETA_RELEASE =
  `https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/${DESKTOP_BETA_TAG}`;

/** Direct asset URL on the current latest (non-prerelease) release. */
export function releaseAsset(filename: string): string {
  return `${REPO_RELEASES_LATEST}/download/${encodeURIComponent(filename)}`;
}

export function desktopBetaAsset(filename: string): string {
  return `https://github.com/LEONINE-DAO/Nozy-wallet/releases/download/${DESKTOP_BETA_TAG}/${encodeURIComponent(filename)}`;
}

/** Production CLI assets (attached on every CLI tag). */
export const DOWNLOAD_URLS = {
  cliWindows: releaseAsset("nozy-windows.exe"),
  cliLinux: releaseAsset("nozy-linux"),
  cliMacIntel: releaseAsset("nozy-macos-intel"),
  cliMacArm: releaseAsset("nozy-macos-arm"),
  hashes: releaseAsset("HASHES.txt"),
} as const;

/** Desktop beta.2 installer / binaries (prerelease). */
export const DESKTOP_DOWNLOAD_URLS = {
  windows: desktopBetaAsset("nozy-desktop-windows-x86_64-installer.exe"),
  linux: desktopBetaAsset("nozy-desktop-linux-x86_64.tar.gz"),
  macArm: desktopBetaAsset("nozy-desktop-macos-aarch64.tar.gz"),
  releasePage: DESKTOP_BETA_RELEASE,
} as const;
