export type WalletInfo = {
  exists: boolean;
  has_password: boolean;
};

export type BalanceResponse = {
  balance_zec: number;
  balance_zatoshis: number;
};

export type SyncResponse = {
  success: boolean;
  balance_zec: number;
  balance_zatoshis: number;
  notes_found: number;
  total_notes: number;
  new_notes_in_scan: number;
  last_scan_height: number;
  chain_tip: number;
  already_synced: boolean;
  message: string;
};

export type CreateWalletResponse = {
  mnemonic: string;
};

export type AddressResponse = {
  address: string;
};

export type SendTransactionResponse = {
  success: boolean;
  txid: string | null;
  message: string;
};

export type FeeEstimateResponse = {
  fee_zatoshis: number;
  fee_zec: number;
  priority: boolean;
};

export type ConfigResponse = {
  zebra_url: string;
  network: string;
  last_scan_height: number | null;
  theme: string;
};

export type TransactionRecord = {
  txid: string;
  status: string;
  amount_zec: number;
  fee_zec: number;
  recipient: string;
  block_height: number | null;
  confirmations: number | null;
  broadcast_at: string | null;
  memo: string | null;
};

export type TransactionHistoryResponse = {
  transactions: TransactionRecord[];
  total: number;
};

export type AddressBookEntry = {
  name: string;
  address: string;
  created_at: string;
  last_used: string | null;
  usage_count: number;
  notes: string | null;
};

export type KeystoneStatusResponse = {
  enabled: boolean;
  device_label: string | null;
  has_ufvk: boolean;
  pending_send: boolean;
};

export type KeystonePrepareResponse = {
  success: boolean;
  summary?: string;
  action_count?: number;
  pczt_hex?: string;
  ur_frames?: string[];
  ur_type?: string;
  message?: string;
};

export type WalletStatusResponse = {
  balance_zec: number;
  pending_transactions: number;
  total_transactions: number;
  last_sync_height: number | null;
  current_block_height: number | null;
  blocks_behind: number | null;
};

export type ApiError = {
  error: string;
  message?: string;
  code?: string;
};

export type RootStackParamList = {
  Welcome: undefined;
  CreateWallet: undefined;
  MnemonicBackup: { mnemonic: string };
  RestoreWallet: undefined;
  Unlock: undefined;
  Dashboard: undefined;
  Send: { recipient?: string } | undefined;
  TransactionHistory: undefined;
  TransactionDetail: { txid: string };
  Settings: undefined;
  About: undefined;
  AddressBook: undefined;
  Keystone: undefined;
};
