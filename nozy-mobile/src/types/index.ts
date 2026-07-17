export type WalletInfo = {
  exists: boolean;
  has_password: boolean;
};

export type BalanceResponse = {
  /** Legacy alias for confirmed; prefer `available_zec` for spendable UI. */
  balance_zec: number;
  balance_zatoshis: number;
  confirmed_zec?: number;
  confirmed_zatoshis?: number;
  pending_zec?: number;
  pending_zatoshis?: number;
  available_zec?: number;
  available_zatoshis?: number;
  unspent_note_count?: number;
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
  expiry_delta_blocks?: number;
  fee_source?: string;
  estimated_at?: string;
};

export type CheckConfirmationsResponse = {
  pending_updated: number;
  expired_updated: number;
  confirmations_updated: number;
};

export type SpeedUpTransactionResponse = {
  success: boolean;
  txid: string | null;
  original_txid: string;
  message: string;
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
  network: string;
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
  balance_zatoshis?: number;
  confirmed_zec?: number;
  pending_zec?: number;
  available_zec?: number;
  pending_transactions: number;
  total_transactions: number;
  last_sync_height: number | null;
  current_block_height: number | null;
  blocks_behind: number | null;
  witness_lag_blocks: number;
  witness_fresh_for_send: boolean;
  max_send_witness_lag_blocks: number;
  ready_for_send: boolean;
};

export type WalletProfileInfo = {
  id: string;
  name: string;
  created_at: number;
  has_wallet: boolean;
  is_active: boolean;
  network: string;
  zebra_url: string;
};

export type NetworkWalletStatusResponse = {
  network: string;
  zebra_url: string;
  active_profile: WalletProfileInfo | null;
  profiles: WalletProfileInfo[];
  suggested_testnet_profile_id: string | null;
  testnet_ready: boolean;
};

export type TestnetWalletResponse = {
  profile: WalletProfileInfo;
  address: string;
  mnemonic: string | null;
};

export type PrivacyNetworkResponse = {
  tor_enabled: boolean;
  tor_proxy: string;
  i2p_enabled: boolean;
  i2p_proxy: string;
  preferred_network: string;
  require_privacy_network: boolean;
  broadcast_via_nym_mixnet: boolean;
  attest_private_network: boolean;
  force_clearnet: boolean;
};

export type IronwoodSaferMigrationStatus = {
  network_privacy_allowed: boolean;
  network_privacy_mode: string | null;
  zebra_url_local: boolean;
  privacy_proxy_detected: boolean;
  privacy_proxy_label: string | null;
  user_attested: boolean;
  force_clearnet: boolean;
  network_privacy_blockers: string[];
  network_privacy_warnings: string[];
  cover_bucket_height: number;
  cover_local_transfers: number;
  cover_k_max: number;
  cover_thin: boolean;
  cover_warnings: string[];
  cover_notes: string[];
  amount_timing_active: string;
  amount_timing_planned: string;
  amount_timing_notes: string[];
};

export type IronwoodStatusResponse = {
  network: string;
  chain_tip: number | null;
  activation_height: number | null;
  activation_target_date: string;
  ironwood_active: boolean;
  ironwood_rpc_detected: boolean;
  orchard_chain_value_zec: number | null;
  ironwood_chain_value_zec: number | null;
  orchard_wallet_zat: number;
  ironwood_wallet_zat: number;
  ironwood_send_enabled: boolean;
  wallet_ready: boolean;
  migration_recommended: boolean;
  migration_note_count: number;
  migration_zat: number;
  zip318_transfer_count: number;
  zip318_note_split_required: boolean;
  next_anchor_bucket_height: number | null;
  migration_enabled: boolean;
  blockers: string[];
  activation_notice: string;
  migration_privacy_warnings: string[];
  orchard_funds_at_risk: boolean;
  safer_migration: IronwoodSaferMigrationStatus;
};

export type IronwoodPlanSaveResponse = {
  orchard_notes_to_migrate: number;
  total_zatoshis: number;
  transfer_count: number;
  note_split_required: boolean;
  next_anchor_bucket_height: number | null;
  schedule_path: string;
  ironwood_active: boolean;
  message: string;
};

export type IronwoodSplitResponse = {
  dry_run: boolean;
  source_value_zat: number;
  source_nullifier_hex: string;
  fee_zat: number;
  output_values_zat: number[];
  txid: string | null;
  note_split_still_required: boolean;
  message: string;
};

export type IronwoodMigrateResponse = {
  readiness_state: string;
  orchard_notes_to_migrate: number;
  total_zatoshis: number;
  total_transfer_count: number;
  schedule_path: string | null;
  prepared_txid: string | null;
  prepared_sequence: number | null;
  prepared_value_zat: number | null;
  prepared_at_height: number | null;
  expires_at_height: number | null;
  blockers: string[];
  message: string;
};

export type IronwoodBroadcastResponse = {
  readiness_state: string;
  sequence: number;
  txid: string;
  broadcast_at_height: number;
  schedule_path: string;
  confirmed: boolean;
  blockers: string[];
  message: string;
};

export type ProvingStatusResponse = {
  spend_params: boolean;
  output_params: boolean;
  spend_vk: boolean;
  output_vk: boolean;
  can_prove: boolean;
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
  Ironwood: undefined;
};
