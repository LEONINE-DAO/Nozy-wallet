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
}

export interface SendTransactionRequest {
  recipient: string;
  amount: number;
  memo?: string;
  password?: string;
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

/** Address book entry (matches backend AddressEntry). */
export interface AddressBookEntry {
  name: string;
  address: string;
  created_at: string;
  last_used?: string | null;
  usage_count: number;
  notes?: string | null;
}

/** Request to add an address book entry. */
export interface AddAddressBookRequest {
  name: string;
  address: string;
  notes?: string | null;
}
