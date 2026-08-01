import Constants from "expo-constants";

type Extra = {
  enableExperimentalFeatures?: boolean;
  requireHostedApiKey?: boolean;
};

function extra(): Extra {
  return (Constants.expoConfig?.extra ?? {}) as Extra;
}

/** True for store / standalone release builds. */
export function isProductionBuild(): boolean {
  const env = Constants.executionEnvironment;
  return env === "storeClient" || env === "standalone";
}

/** Store builds hide on-device + light-client settings unless extra overrides. */
export function enableExperimentalFeatures(): boolean {
  if (extra().enableExperimentalFeatures === true) return true;
  if (extra().enableExperimentalFeatures === false) return false;
  return !isProductionBuild();
}

export function requireHostedApiKey(): boolean {
  return extra().requireHostedApiKey !== false;
}
