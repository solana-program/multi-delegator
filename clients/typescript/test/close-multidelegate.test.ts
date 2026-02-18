import { describe, expect, test } from 'vitest';
import {
  fetchMaybeMultiDelegate,
  fetchMultiDelegate,
} from '../src/generated/index.ts';
import { getMultiDelegatePDA } from '../src/pdas.ts';
import { DEFAULT_TEST_BALANCE, initTestSuite } from './setup.ts';

describe('MultiDelegator Close MultiDelegate Tests', () => {
  test('close multi delegate returns rent to user', async () => {
    const testSuite = await initTestSuite();

    const userAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      testSuite.payer.address,
      DEFAULT_TEST_BALANCE,
    );

    await testSuite.client.initMultiDelegate(
      testSuite.payer,
      testSuite.tokenMint,
      userAta,
      testSuite.tokenProgram,
    );

    const [multiDelegatePda] = await getMultiDelegatePDA(
      testSuite.payer.address,
      testSuite.tokenMint,
    );

    // Verify account exists before closing
    const accountBefore = await fetchMultiDelegate(
      testSuite.rpc,
      multiDelegatePda,
    );
    expect(accountBefore).toBeDefined();
    expect(accountBefore.data.user).toBe(testSuite.payer.address);
    const accountRent = accountBefore.lamports;

    const balanceBefore = await testSuite.rpc
      .getBalance(testSuite.payer.address)
      .send();

    // Close the multi delegate
    const result = await testSuite.client.closeMultiDelegate(
      testSuite.payer,
      testSuite.tokenMint,
    );

    expect(result.signature).toBeDefined();

    // Verify account no longer exists
    const accountAfter = await fetchMaybeMultiDelegate(
      testSuite.rpc,
      multiDelegatePda,
    );
    expect(accountAfter.exists).toBe(false);

    // Verify rent was returned to user
    const balanceAfter = await testSuite.rpc
      .getBalance(testSuite.payer.address)
      .send();
    expect(balanceAfter.value).toBeGreaterThan(balanceBefore.value);
    expect(balanceAfter.value).toBeGreaterThanOrEqual(
      balanceBefore.value + accountRent - 10000n,
    );
  });

  test('non-owner cannot close multi delegate', async () => {
    const testSuite = await initTestSuite();

    const userAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      testSuite.payer.address,
      DEFAULT_TEST_BALANCE,
    );

    await testSuite.client.initMultiDelegate(
      testSuite.payer,
      testSuite.tokenMint,
      userAta,
      testSuite.tokenProgram,
    );

    // Create a different funded keypair that is NOT the owner
    const nonOwner = await testSuite.createFundedKeypair();

    // Attempt to close with the non-owner should fail
    await expect(
      testSuite.client.closeMultiDelegate(nonOwner, testSuite.tokenMint),
    ).rejects.toThrow();

    // Verify account still exists
    const [multiDelegatePda] = await getMultiDelegatePDA(
      testSuite.payer.address,
      testSuite.tokenMint,
    );
    const account = await fetchMultiDelegate(testSuite.rpc, multiDelegatePda);
    expect(account).toBeDefined();
    expect(account.data.user).toBe(testSuite.payer.address);
  });
});
