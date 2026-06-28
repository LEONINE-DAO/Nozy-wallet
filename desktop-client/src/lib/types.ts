export interface ApiResponse<T> {
  data: T;
  status: number;
}

export interface WalletExistsResponse {
  exists: boolean;
}

export interface CreateWalletRequest {
  password?: string;
}

export interface RestoreWalletRequest {
  mnemonic: string;
  password?: string;
}

export interface UnlockWalletRequest {
  password: string;
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

export interface SendTransactionRequest {
  recipient: string;
  amount: number;
  memo?: string;
  password?: string;
  /** Pilot: ZIP-317 standard fee × 4 when true (default false). */
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
