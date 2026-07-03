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
