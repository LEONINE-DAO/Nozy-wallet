type ApiRequest = {
  method: string;
  params?: Record<string, unknown>;
};

type ApiResponse<T> = {
  result: T | null;
  error: { message: string } | null;
};

export type WalletStatus = {
  exists: boolean;
  unlocked: boolean;
  address: string | null;
  rpcEndpoint: string;
};

export type PendingApproval = {
  id: string;
  kind: "sign" | "transaction";
  payload: Record<string, unknown>;
  createdAt: number;
};

function sendMessage<T>(request: ApiRequest): Promise<T> {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(
      {
        type: "NOZY_REQUEST",
        method: request.method,
        params: request.params ?? {}
      },
      (response: ApiResponse<T>) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
          return;
        }
        if (!response) {
          reject(new Error("No response from Nozy background worker"));
          return;
        }
        if (response.error) {
          reject(new Error(response.error.message));
          return;
        }
        resolve(response.result as T);
      }
    );
  });
}

export const extensionApi = {
  walletStatus: () => sendMessage<WalletStatus>({ method: "wallet_status" }),
  walletCreate: (password: string) =>
    sendMessage<{ address: string }>({ method: "wallet_create", params: { password } }),
  walletRestore: (mnemonic: string, password: string) =>
    sendMessage<{ address: string }>({
      method: "wallet_restore",
      params: { mnemonic, password }
    }),
  walletUnlock: (password: string) =>
    sendMessage<{ address: string }>({ method: "wallet_unlock", params: { password } }),
  walletLock: () => sendMessage<boolean>({ method: "wallet_lock" }),
  walletGenerateAddress: (account = 0, index = 0) =>
    sendMessage<string>({
      method: "wallet_generate_address",
      params: { account, index }
    }),
  walletSignMessage: (message: string) =>
    sendMessage<string>({
      method: "wallet_sign_message",
      params: { message }
    }),
  walletGetPendingApprovals: () =>
    sendMessage<PendingApproval[]>({ method: "wallet_get_pending_approvals" }),
  walletApproveRequest: (id: string) =>
    sendMessage<{ approved: boolean; id: string }>({
      method: "wallet_approve_request",
      params: { id }
    }),
  walletRejectRequest: (id: string) =>
    sendMessage<{ approved: boolean; id: string }>({
      method: "wallet_reject_request",
      params: { id }
    }),
  rpcSetEndpoint: (url: string) =>
    sendMessage<{ rpcEndpoint: string }>({
      method: "rpc_set_endpoint",
      params: { url }
    }),
  rpcGetStatus: () =>
    sendMessage<{ endpoint: string; connected: boolean }>({ method: "rpc_get_status" }),
  rpcGetBlockCount: () => sendMessage<number>({ method: "rpc_get_block_count" })
};

