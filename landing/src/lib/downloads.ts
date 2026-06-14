/**
 * CLI binaries ship with every release tag. Other surfaces (desktop, extension)
 * are in development and are not linked for end-user download yet.
 */
export const REPO_RELEASES_LATEST =
  "https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest";

/** Direct asset URL when you know the file exists on the current latest release. */
export function releaseAsset(filename: string): string {
  return `${REPO_RELEASES_LATEST}/download/${encodeURIComponent(filename)}`;
}

/** Production-ready CLI assets (attached on every tag). */
export const DOWNLOAD_URLS = {
  cliWindows: releaseAsset("nozy-windows.exe"),
  cliLinux: releaseAsset("nozy-linux"),
  cliMacIntel: releaseAsset("nozy-macos-intel"),
  cliMacArm: releaseAsset("nozy-macos-arm"),
  hashes: releaseAsset("HASHES.txt"),
} as const;
