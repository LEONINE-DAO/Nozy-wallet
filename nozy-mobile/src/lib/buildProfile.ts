import Constants from "expo-constants";

type Extra = {
  requireHostedApiKey?: boolean;
};

function extra(): Extra {
  return (Constants.expoConfig?.extra ?? {}) as Extra;
}

export function requireHostedApiKey(): boolean {
  return extra().requireHostedApiKey === true;
}
