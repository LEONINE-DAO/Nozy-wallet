import { rpcCall, isPublicLwdEndpoint } from './rpc-utils.js';

export async function getBlockCount(session) {
  if (isPublicLwdEndpoint(session.rpcEndpoint)) {
    // For public LWD endpoints, query via lightwalletd gRPC/REST or companion API fallback
    return await session.companionApi.getBlockCount();
  }
  const res = await rpcCall(session, 'getblockcount', []);
  return res;
}

export async function broadcastRawHex(session, rawHex) {
  if (isPublicLwdEndpoint(session.rpcEndpoint)) {
    return await session.companionApi.sendRawTransaction(rawHex);
  }
  const res = await rpcCall(session, 'sendrawtransaction', [rawHex]);
  return res;
}
