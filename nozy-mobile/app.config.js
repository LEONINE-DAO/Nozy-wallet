/** Dynamic Expo config — production vs development (EAS + local). */
const appJson = require("./app.json");

const PROFILE = process.env.EAS_BUILD_PROFILE ?? "development";
const IS_PRODUCTION =
  PROFILE === "production" ||
  process.env.EXPO_PUBLIC_APP_VARIANT === "production";

/** @returns {import("@expo/config-types").ExpoConfig} */
module.exports = () => {
  const base = appJson.expo;
  return {
    ...base,
    owner: "leonine-dao",
    extra: {
      ...base.extra,
      appVariant: IS_PRODUCTION ? "production" : "development",
      easBuildProfile: PROFILE,
      requireHostedApiKey: false,
      defaultApiUrl: base.extra.defaultApiUrl,
      hostedLwdUrl:
        base.extra.hostedLwdUrl ?? "https://lwd.nozywallet.org:443",
    },
    // Cleartext HTTP is controlled in android/ (debug manifest), not Expo schema —
    // `usesCleartextTraffic` is invalid on android config and is not synced when android/ exists.
    ios: {
      ...base.ios,
      infoPlist: {
        ...base.ios?.infoPlist,
        ITSAppUsesNonExemptEncryption: false,
      },
    },
  };
};
