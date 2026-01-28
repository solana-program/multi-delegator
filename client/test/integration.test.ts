import { describe, expect, test } from 'bun:test';
import { fetchMultiDelegate } from '../src/generated/index.ts';
import { getMultiDelegatePDA } from '../src/pdas.ts';
import { initTestSuite } from './setup.ts';

describe('MultiDelegator Integration Tests', () => {
  test('can connect to surfpool validator', async () => {
    const testSuite = await initTestSuite();

    const res = await testSuite.rpc.getHealth().send();

    expect(res).toBe('ok');
  });

  test('initialize multi delegate', async () => {
    // Setup - testSuite has payer and tokenMint pre-created
    const testSuite = await initTestSuite();

    // Create user ATA with tokens (individual test creates its own ATA)
    // Maybe we don't even need this? :thinking_face:
    const userAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      testSuite.payer.address,
      1_000_000n, // 1 token with 6 decimals
    );

    // Execute - initialize multi delegate
    const result = await testSuite.client.initMultiDelegate(
      testSuite.payer,
      testSuite.tokenMint,
      userAta,
    );

    expect(result.signature).toBeDefined();

    // Verify MultiDelegate account exists
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
