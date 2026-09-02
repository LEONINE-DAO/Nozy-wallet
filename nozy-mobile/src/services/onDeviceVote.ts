import {
  isNozyWalletNativeAvailable,
  voteCalendarInfo,
  voteExportNotes,
  voteSignDelegation,
  type VoteCalendarNative,
} from "nozy-wallet";
import { loadOnDevicePaths } from "./onDeviceSapling";

export async function onDeviceVoteCalendar(): Promise<VoteCalendarNative> {
  return voteCalendarInfo();
}

export async function onDeviceVoteExportNotes(network = "mainnet"): Promise<{
  note_count: number;
  total_value_zat: number;
  notes_json: string;
  message: string;
}> {
  if (!isNozyWalletNativeAvailable()) {
    throw new Error(
      "On-device vote export needs a native build with libnozy_ffi. Companion users: use Desktop Vote.",
    );
  }
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error(
      "Configure on-device mnemonic and data paths in Settings → On-device wallet.",
    );
  }
  const res = await voteExportNotes({
    mnemonic: paths.mnemonic,
    walletDataDir: paths.walletDataDir,
    network,
  });
  return {
    note_count: res.note_count,
    total_value_zat: res.total_value_zat,
    notes_json: res.notes_json,
    message: res.message,
  };
}

export async function onDeviceVoteSignDelegation(requestJson: string): Promise<{
  round_id: string;
  sig_json: string;
  message: string;
}> {
  if (!isNozyWalletNativeAvailable()) {
    throw new Error(
      "On-device vote sign needs a native build with libnozy_ffi. Companion users: use Desktop Vote.",
    );
  }
  const paths = await loadOnDevicePaths();
  if (!paths) {
    throw new Error(
      "Configure on-device mnemonic and data paths in Settings → On-device wallet.",
    );
  }
  const res = await voteSignDelegation({
    mnemonic: paths.mnemonic,
    requestJson,
  });
  return {
    round_id: res.round_id,
    sig_json: res.sig_json,
    message: res.message,
  };
}
