import { create } from "zustand";

export interface TokenDetail {
  id: string;
  name: string;
  symbol: string;
  decimals: number;
  icon?: string;
  isNative: boolean;
}

interface TokenState {
  tokens: TokenDetail[];
  activeTokenId: string | null;
  setTokens: (tokens: TokenDetail[]) => void;
  setActiveToken: (tokenId: string) => void;
  getToken: (id: string) => TokenDetail | undefined;
}

const DEFAULT_TOKENS: TokenDetail[] = [
  {
    id: "zcash",
    name: "Zcash",
    symbol: "ZEC",
    decimals: 8,
    isNative: true,
  },
];

export const useTokenStore = create<TokenState>((set, get) => ({
  tokens: DEFAULT_TOKENS,
  activeTokenId: "zcash",
  setTokens: (tokens) => set({ tokens }),
  setActiveToken: (tokenId) => set({ activeTokenId: tokenId }),
  getToken: (id) => get().tokens.find((t) => t.id === id),
}));
