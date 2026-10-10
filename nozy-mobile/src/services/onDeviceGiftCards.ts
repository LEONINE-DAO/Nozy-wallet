import { Platform } from "react-native";
import {
  giftCardBuy,
  giftCardCancel,
  giftCardList,
  giftCardRedeem,
} from "nozy-wallet";
import { loadOnDevicePaths } from "./onDeviceSapling";
import type { GiftCard, GiftCardCreated } from "../types";

function zebraFromPhone(url: string): string {
  const trimmed = url.trim() || "http://127.0.0.1:8232";
  if (Platform.OS !== "android") return trimmed;
  return trimmed
    .replace("://127.0.0.1", "://10.0.2.2")
    .replace("://localhost", "://10.0.2.2");
}

function parseCard(raw: string): GiftCard {
  return JSON.parse(raw) as GiftCard;
}

export async function listOnDeviceGiftCards(): Promise<GiftCard[]> {
  const paths = await loadOnDevicePaths();
  if (!paths) return [];
  const raw = await giftCardList(paths.walletDataDir, 20);
  const parsed = JSON.parse(raw) as GiftCard[] | { gift_cards?: GiftCard[] };
  if (Array.isArray(parsed)) return parsed;
  return parsed.gift_cards ?? [];
}

export async function buyOnDeviceGiftCard(params: {
  amountZec: number;
  label?: string;
}): Promise<GiftCardCreated> {
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error("No wallet on this phone. Create or restore one first.");
  }
  const raw = await giftCardBuy({
    mnemonic: paths.mnemonic,
    walletDataDir: paths.walletDataDir,
    zebraUrl: zebraFromPhone(paths.zebraUrl),
    amountZec: params.amountZec,
    label: params.label,
  });
  return JSON.parse(raw) as GiftCardCreated;
}

export async function redeemOnDeviceGiftCard(
  claimCode: string,
): Promise<GiftCard> {
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error("No wallet on this phone. Create or restore one first.");
  }
  const raw = await giftCardRedeem({
    mnemonic: paths.mnemonic,
    walletDataDir: paths.walletDataDir,
    zebraUrl: zebraFromPhone(paths.zebraUrl),
    claimCode,
  });
  return parseCard(raw);
}

export async function cancelOnDeviceGiftCard(giftId: string): Promise<GiftCard> {
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error("No wallet on this phone. Create or restore one first.");
  }
  return parseCard(await giftCardCancel(paths.walletDataDir, giftId));
}
