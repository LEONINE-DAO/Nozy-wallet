export const BOOK = "https://leonine-dao.github.io/Nozy-wallet/book/";
export const MANIFESTO = `${BOOK}nozy/manifesto.html`;
export const REPO = "https://github.com/LEONINE-DAO/Nozy-wallet";
export const REPO_RELEASES = `${REPO}/releases/latest`;

export const PATHS = {
  enhancementRoadmap: `${REPO}/blob/master/ENHANCEMENT_ROADMAP.md`,
  webApp: `${REPO}/blob/master/web-app/README.md`,
  cli: `${REPO}#what-nozywallet-is`,
  desktop: `${REPO}/tree/master/desktop-client`,
  extension: `${REPO}/tree/master/browser-extension`,
  extensionCompanion: `${REPO}/blob/master/browser-extension/COMPANION.md`,
  mobile: `${REPO}/tree/master/nozy-mobile`,
  mobileReadme: `${REPO}/blob/master/nozy-mobile/README.md`,
  apiServer: `${REPO}/tree/master/api-server`,
  operatorDeploy: `${REPO}/blob/master/nozy-mobile/VPS-DEPLOY.md`,
  multichainRfc: `${REPO}/blob/master/docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md`,
} as const;
