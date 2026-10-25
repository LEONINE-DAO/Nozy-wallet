import { getBlockCount } from './background/tx-utils.js';
import assert from 'assert';

async function testGetBlockCountPublicLwd() {
  const session = {
    rpcEndpoint: 'https://lwd.nozywallet.org:443',
    companionApi: {
      async getBlockCount() {
        return 123456;
      }
    }
  };
  const count = await getBlockCount(session);
  assert.strictEqual(count, 123456);
  console.log('Test passed successfully');
}

testGetBlockCountPublicLwd();
