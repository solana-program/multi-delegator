import { generateKeyPairSigner } from 'gill';
import { describe, expect, test } from 'vitest';
import { fetchFixedDelegation } from '../src/generated/index.ts';
import { getDelegationPDA, getMultiDelegatePDA } from '../src/pdas.ts';
import {
  DEFAULT_TEST_BALANCE,
  initTestSuite,
  ONE_HOUR_IN_SECONDS,
} from './setup.ts';

describe('MultiDelegator Fixed Delegation Tests', () => {
  test('create fixed delegation', async () => {
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

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;
    const amount = 500_000n;
    const expiryS = BigInt(Math.floor(Date.now() / 1000) + ONE_HOUR_IN_SECONDS);

    const result = await testSuite.client.createFixedDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amount,
      expiryS,
    );

    expect(result.signature).toBeDefined();

    const [multiDelegatePda] = await getMultiDelegatePDA(
      testSuite.payer.address,
      testSuite.tokenMint,
    );

    const [delegationPda] = await getDelegationPDA(
      multiDelegatePda,
      testSuite.payer.address,
      delegatee.address,
      nonce,
    );

    const delegationAccount = await fetchFixedDelegation(
      testSuite.client.client.rpc,
      delegationPda,
    );

    expect(delegationAccount).toBeDefined();
    expect(delegationAccount.data).toBeDefined();
    expect(delegationAccount.data.amount).toBe(amount);
    expect(delegationAccount.data.expiryTs).toBe(expiryS);
  });

  test('revoke fixed delegation', async () => {
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

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;

    await testSuite.client.createFixedDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      500_000n,
      BigInt(Math.floor(Date.now() / 1000) + ONE_HOUR_IN_SECONDS),
    );

    const [multiDelegatePda] = await getMultiDelegatePDA(
      testSuite.payer.address,
      testSuite.tokenMint,
    );
    const [delegationPda] = await getDelegationPDA(
      multiDelegatePda,
      testSuite.payer.address,
      delegatee.address,
      nonce,
    );

    const delegationBefore = await fetchFixedDelegation(
      testSuite.rpc,
      delegationPda,
    );
    expect(delegationBefore).toBeDefined();
    const delegationRent = delegationBefore.lamports;

    const delegatorBalanceBefore = await testSuite.rpc
      .getBalance(testSuite.payer.address)
      .send();

    const result = await testSuite.client.revokeDelegation(
      testSuite.payer,
      delegationPda,
    );

    expect(result.signature).toBeDefined();

    await expect(
      fetchFixedDelegation(testSuite.rpc, delegationPda),
    ).rejects.toThrow();

    const delegatorBalanceAfter = await testSuite.rpc
      .getBalance(testSuite.payer.address)
      .send();
    expect(delegatorBalanceAfter.value).toBeGreaterThan(
      delegatorBalanceBefore.value,
    );
    expect(delegatorBalanceAfter.value).toBeGreaterThanOrEqual(
      delegatorBalanceBefore.value + delegationRent - 10000n,
    );
  });

  test('transfer fixed delegation', async () => {
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

    const delegatee = await testSuite.createFundedKeypair();
    const nonce = 0n;
    const amount = 500_000n;
    const expiryS = BigInt(Math.floor(Date.now() / 1000) + ONE_HOUR_IN_SECONDS);

    await testSuite.client.createFixedDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amount,
      expiryS,
    );

    const [multiDelegatePda] = await getMultiDelegatePDA(
      testSuite.payer.address,
      testSuite.tokenMint,
    );
    const [delegationPda] = await getDelegationPDA(
      multiDelegatePda,
      testSuite.payer.address,
      delegatee.address,
      nonce,
    );

    // Create a destination ATA for the delegatee
    const delegateeAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      delegatee.address,
      0n,
    );

    // Transfer
    const transferAmount = 100_000n;
    const result = await testSuite.client.transferFixed(
      delegatee,
      testSuite.payer.address,
      userAta,
      testSuite.tokenMint,
      delegationPda,
      transferAmount,
      delegateeAta,
      testSuite.tokenProgram,
    );

    expect(result.signature).toBeDefined();

    // Check balance of delegatee
    const balance = await testSuite.rpc
      .getTokenAccountBalance(delegateeAta)
      .send();
    expect(balance.value.amount).toBe(transferAmount.toString());

    // Check delegation state updated
    const delegationAccount = await fetchFixedDelegation(
      testSuite.rpc,
      delegationPda,
    );
    expect(delegationAccount.data.amount).toBe(amount - transferAmount);
  });
});
