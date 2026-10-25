import { isPublicLwdEndpoint, getBlockCountCompatible } from './tx-utils.js';

describe('tx-utils public sync compatibility', () => {
  test('isPublicLwdEndpoint identifies lwd.nozywallet.org correctly', () => {
    expect(isPublicLwdEndpoint('https://lwd.nozywallet.org:443')).toBe(true);
    expect(isPublicLwdEndpoint('http://127.0.0.1:18232')).toBe(false);
  });

  test('getBlockCountCompatible throws or handles LWD appropriately', async () => {
    const endpoint = 'https://lwd.nozywallet.org:443';
    const mockRpcCall = jest.fn();
    await expect(getBlockCountCompatible(endpoint, mockRpcCall)).rejects.toThrow(
      /Public LWD does not support JSON-RPC getblockcount/
    );
    expect(mockRpcCall).not.toHaveBeenCalled();
  });
});
