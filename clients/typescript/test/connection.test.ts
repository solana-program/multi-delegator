import { describe, expect, test } from 'vitest';
import { initTestSuite } from './setup.ts';

describe('MultiDelegator Connection Tests', () => {
  test('can connect to surfpool validator', async () => {
    const testSuite = await initTestSuite();
    const res = await testSuite.rpc.getHealth().send();
    expect(res).toBe('ok');
  });
});
