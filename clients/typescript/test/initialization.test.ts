import { describe, expect, test } from 'bun:test';
import { fetchMultiDelegate } from '../src/generated/index.ts';
import { getMultiDelegatePDA } from '../src/pdas.ts';
import { DEFAULT_TEST_BALANCE, initTestSuite } from './setup.ts';

describe('MultiDelegator Initialization Tests', () => {
  test('initialize multi delegate', async () => {
    const testSuite = await initTestSuite();

    const userAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      testSuite.payer.address,
      DEFAULT_TEST_BALANCE,
    );

    const result = await testSuite.client.initMultiDelegate(
      testSuite.payer,
      testSuite.tokenMint,
      userAta,
    );

    expect(result.signature).toBeDefined();

    const [multiDelegatePda] = await getMultiDelegatePDA(
      testSuite.payer.address,
      testSuite.tokenMint,
    );

    const multiDelegateAccount = await fetchMultiDelegate(
      testSuite.rpc,
      multiDelegatePda,
    );

    expect(multiDelegateAccount).toBeDefined();
    expect(multiDelegateAccount.data.user).toBe(testSuite.payer.address);
    expect(multiDelegateAccount.data.tokenMint).toBe(testSuite.tokenMint);
  });
});
