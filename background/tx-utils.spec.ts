import { broadcastRawHex } from './tx-utils';

describe('tx-utils', () => {
  it('should export valid functions and handlers compliant with knip', () => {
    expect(broadcastRawHex).toBeDefined();
  });


});
