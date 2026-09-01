export const BOOK = "https://leonine-dao.github.io/Nozy-wallet/book/";
export const MANIFESTO = `${BOOK}nozy/manifesto.html`;
export const REPO = "https://github.com/LEONINE-DAO/Nozy-wallet";
export const REPO_RELEASES = `${REPO}/releases/latest`;

/** Chrome Web Store listing (NozyWallet). */
export const EXTENSION_CHROME_STORE =
  "https://chromewebstore.google.com/detail/nozywallet/pnlmnkallkmelflckjkmohemibfahoce";

/** Optional GitHub zip for load-unpacked / contributor builds. */
export const EXTENSION_RELEASE =
  `${REPO}/releases/tag/extension-v0.1.10`;

export const IRONWOOD_ZODL = "https://ironwood.zodl.com/";
export const IRONWOOD_CIPHERSCAN = "https://cipherscan.app/ironwood";
export const ZIP_318 = "https://zips.z.cash/zip-0318";
export const ZIP_258 = "https://zips.z.cash/zip-0258";
export const IRONWOOD_SHIELDED_LABS = "https://shieldedlabs.net/ironwood/";

export const PATHS = {
  enhancementRoadmap: `${REPO}/blob/master/ENHANCEMENT_ROADMAP.md`,
  webApp: `${REPO}/blob/master/web-app/README.md`,
  /** Live once W4 deploys; local: `cd web-app && npm run dev` → :5174 */
  webAppLive: "https://leonine-dao.github.io/Nozy-wallet/app/",
  cli: `${REPO}#what-nozywallet-is`,
  desktop: `${REPO}/tree/master/desktop-client`,
  extension: `${REPO}/tree/master/browser-extension`,
  extensionCompanion: `${REPO}/blob/master/browser-extension/COMPANION.md`,
  /** Primary install — Chrome Web Store */
  extensionRelease: EXTENSION_CHROME_STORE,
  extensionZip: EXTENSION_RELEASE,
  mobile: `${REPO}/tree/master/nozy-mobile`,
  mobilePage: "https://leonine-dao.github.io/Nozy-wallet/mobile",
  mobileReadme: `${REPO}/blob/master/nozy-mobile/README.md`,
  apiServer: `${REPO}/tree/master/api-server`,
  operatorDeploy: `${REPO}/blob/master/nozy-mobile/VPS-DEPLOY.md`,
  multichainRfc: `${REPO}/blob/master/docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md`,
  ironwoodReadiness: `${REPO}/blob/master/docs/reference/IRONWOOD_WALLET_READINESS.md`,
  ironwoodPage: "/ironwood",
} as const;

