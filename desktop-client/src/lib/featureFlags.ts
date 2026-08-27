/** Web3 Browser tab (iframe + injected provider). Off by default for beta. */
export const dappBrowserEnabled =
  import.meta.env.VITE_ENABLE_DAPP_BROWSER === "true";

/** Watch-only web wallet tab. Off by default. */
export const webWatchOnlyEnabled =
  import.meta.env.VITE_ENABLE_WEB_WATCH_ONLY === "true";

/**
 * NU7 coinholder vote tab (Valar). On by default through the vote window;
 * set `VITE_ENABLE_NU7_VOTE=false` to hide.
 */
export const nu7VoteEnabled =
  import.meta.env.VITE_ENABLE_NU7_VOTE !== "false";
