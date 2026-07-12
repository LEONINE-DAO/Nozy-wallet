export interface ApiResponse<T> {
  data: T;
  status: number;
}

export interface WalletExistsResponse {
  exists: boolean;
  has_password: boolean;
}

export interface CreateWalletRequest {
  password?: string;
  name?: string;
}

export interface WalletProfileInfo {
  id: string;
  name: string;
  created_at: number;
  has_wallet: boolean;
  is_active: boolean;
  network: string;
  zebra_url: string;
}

export interface NetworkWalletStatusResponse {
  network: string;
  zebra_url: string;
  active_profile: WalletProfileInfo | null;
  profiles: WalletProfileInfo[];
  suggested_testnet_profile_id: string | null;
  testnet_ready: boolean;
}

export interface ConfigureNetworkWalletRequest {
  network: "mainnet" | "testnet";
  profile_id?: string | null;
  zebra_url?: string | null;
}

export interface DesktopTestnetWalletRequest {
  name?: string;
  password?: string;
  mnemonic?: string;
  rpc_url?: string;
}

export interface DesktopTestnetWalletResponse {
  profile: WalletProfileInfo;
  address: string;
  mnemonic: string | null;
}

export interface RestoreWalletRequest {
  mnemonic: string;
  password?: string;
}

export interface UnlockWalletRequest {
  password?: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface GenerateAddressResponse {
  address: string;
}

export interface BalanceResponse {
  balance: number;
  verified_balance: number;
  confirmed_zec?: number;
  pending_zec?: number;
  available_zec?: number;
  unspent_note_count?: number;
}

export interface SyncStatusResponse {
  zebra_tip: number | null;
  last_scan_height: number | null;
  scan_gap_blocks: number | null;
  witness_lag_blocks: number;
  witness_fresh_for_send: boolean;
  max_send_witness_lag_blocks: number;
  lightwalletd_url: string;
  lwd_tip: number | null;
  lwd_error: string | null;
  compact_max_height: number | null;
  compact_db_exists: boolean;
    message: string;
}

export interface OrchardPoolStatsResponse {
  chain_value_zec: number;
  chain_value_zat: number;
  monitored: boolean;
  block_height: number;
}

export interface IronwoodSaferMigrationStatus {
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
}

export interface IronwoodStatusRequest {
  attest_private_network?: boolean;
  force_clearnet?: boolean;
}

export interface IronwoodDesktopStatusResponse {
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
  readiness_state: string;
  ready_to_prebuild: boolean;
  ready_to_broadcast: boolean;
  blockers: string[];
  safer_migration: IronwoodSaferMigrationStatus;
}

export interface IronwoodPlanSaveResponse {
  orchard_notes_to_migrate: number;
  total_zatoshis: number;
  transfer_count: number;
  note_split_required: boolean;
  next_anchor_bucket_height: number | null;
  schedule_path: string;
  ironwood_active: boolean;
  message: string;
}

export interface IronwoodMigrateRequest {
  password?: string;
}

export interface IronwoodMigrateResponse {
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
}

export interface IronwoodBroadcastRequest {
  password?: string;
  attest_private_network?: boolean;
  force_clearnet?: boolean;
  dry_run?: boolean;
  wait_confirm?: boolean;
}

export interface IronwoodBroadcastResponse {
  readiness_state: string;
  sequence: number;
  txid: string;
  broadcast_at_height: number;
  schedule_path: string;
  confirmed: boolean;
  blockers: string[];
  message: string;
}

export interface IronwoodSplitRequest {
  password?: string;
  dry_run?: boolean;
}

export interface IronwoodSplitResponse {
  dry_run: boolean;
  source_value_zat: number;
  source_nullifier_hex: string;
  fee_zat: number;
  output_values_zat: number[];
  txid: string | null;
  note_split_still_required: boolean;
  message: string;
}

export interface SyncWalletResponse {
  success: boolean;
  balance_zec: number;
  notes_found: number;
  message: string;
  last_scan_height?: number;
  chain_tip?: number;
  already_synced: boolean;
}

export interface SendTransactionRequest {
  recipient: string;
  amount: number;
  memo?: string;
  password?: string;
  /** NozyWallet always uses ZIP-317 × 4; kept for API compat, ignored by core. */
  priority?: boolean;
}

export interface ConfigResponse {
  zebra_url: string;
  theme: string;
}

export interface SetZebraUrlRequest {
  url: string;
}

export interface SetThemeRequest {
  theme: string;
}

export interface ProvingStatusResponse {
  downloaded: boolean;
  progress: number;
}

export interface VerifyPasswordRequest {
  password: string;
}

export interface SignMessageRequest {
  message: string;
  password: string;
}

export interface SignMessageResponse {
  signature: string;
}

export interface AddressBookEntry {
  name: string;
  address: string;
  created_at: string;
  last_used?: string | null;
  usage_count: number;
  notes?: string | null;
}

export interface AddAddressBookRequest {
  name: string;
  address: string;
  notes?: string | null;
}

export interface BackupPathRequest {
  backup_path: string;
}

export interface BackupActionResponse {
  success: boolean;
  path: string;
  message: string;
}

export interface CosignPreparedSend {
  recipient: string;
  amount_zatoshis: number;
  fee_zatoshis: number;
  summary: string;
  action_count: number;
  pczt_hex: string;
  created_at: string;
}

export interface PrepareCosignRequest {
  recipient: string;
  amount: number;
  memo?: string;
  password?: string;
}

export interface PrepareCosignResponse {
  request: CosignPreparedSend;
  ur_frames: string[];
}

export interface SignCosignRequest {
  pczt_hex: string;
  password?: string;
}

export interface CompleteCosignSendRequest {
  pczt_hex: string;
  recipient: string;
  amount: number;
  memo?: string;
  password?: string;
}

export interface KeystoneStatusResponse {
  enabled: boolean;
  device_label: string | null;
  has_ufvk: boolean;
  pending_send: boolean;
  network: string;
}

export interface KeystonePrepareResponse {
  success: boolean;
  summary?: string;
  action_count?: number;
  pczt_hex?: string;
  ur_frames?: string[];
  ur_type?: string;
  message?: string;
}
