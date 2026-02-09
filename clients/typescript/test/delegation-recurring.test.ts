import { describe, expect, test } from 'bun:test';
import { generateKeyPairSigner } from 'gill';
import { fetchRecurringDelegation } from '../src/generated/index.ts';
import { getDelegationPDA, getMultiDelegatePDA } from '../src/pdas.ts';
import {
  DEFAULT_TEST_BALANCE,
  initTestSuite,
  ONE_DAY_IN_SECONDS,
} from './setup.ts';

describe('MultiDelegator Recurring Delegation Tests', () => {
  test('create recurring delegation', async () => {
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
    );

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;
    const amountPerPeriod = 100_000n;
    const periodLengthS = BigInt(ONE_DAY_IN_SECONDS);
    const startTs = BigInt(Math.floor(Date.now() / 1000));
    const expiryS = BigInt(
      Math.floor(Date.now() / 1000) + ONE_DAY_IN_SECONDS * 30,
    );

    const result = await testSuite.client.createRecurringDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amountPerPeriod,
      periodLengthS,
      startTs,
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

    const delegationAccount = await fetchRecurringDelegation(
      testSuite.client.client.rpc,
      delegationPda,
    );

    expect(delegationAccount).toBeDefined();
    expect(delegationAccount.data).toBeDefined();
    expect(delegationAccount.data.expiryTs).toBe(expiryS);
    expect(delegationAccount.data.periodLengthS).toBe(periodLengthS);
    expect(delegationAccount.data.currentPeriodStartTs).toBe(startTs);
    expect(delegationAccount.data.amountPerPeriod).toBe(amountPerPeriod);
    expect(delegationAccount.data.amountPulledInPeriod).toBe(0n);
  });

  test('revoke recurring delegation', async () => {
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
    );

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;

    await testSuite.client.createRecurringDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      100_000n,
      BigInt(ONE_DAY_IN_SECONDS),
      BigInt(Math.floor(Date.now() / 1000)),
      BigInt(Math.floor(Date.now() / 1000) + ONE_DAY_IN_SECONDS * 30),
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

    const delegationBefore = await fetchRecurringDelegation(
      testSuite.rpc,
      delegationPda,
    );
    expect(delegationBefore).toBeDefined();

    const result = await testSuite.client.revokeDelegation(
      testSuite.payer,
      delegationPda,
    );

    expect(result.signature).toBeDefined();

    expect(
      fetchRecurringDelegation(testSuite.rpc, delegationPda),
    ).rejects.toThrow();
  });

  test('transfer recurring delegation', async () => {
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
    );

    const delegatee = await testSuite.createFundedKeypair();
    const nonce = 0n;
    const amountPerPeriod = 100_000n;
    const periodLengthS = BigInt(ONE_DAY_IN_SECONDS);
    const startTs = BigInt(Math.floor(Date.now() / 1000));
    const expiryS = BigInt(
      Math.floor(Date.now() / 1000) + ONE_DAY_IN_SECONDS * 30,
    );

    await testSuite.client.createRecurringDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amountPerPeriod,
      periodLengthS,
      startTs,
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
    const transferAmount = 50_000n;
    const result = await testSuite.client.transferRecurring(
      delegatee,
      testSuite.payer.address,
      userAta,
      testSuite.tokenMint,
      delegationPda,
      transferAmount,
      delegateeAta,
    );

    expect(result.signature).toBeDefined();

    // Check balance of delegatee
    const balance = await testSuite.rpc
      .getTokenAccountBalance(delegateeAta)
      .send();
    expect(balance.value.amount).toBe(transferAmount.toString());

    // Check delegation state updated
    const delegationAccount = await fetchRecurringDelegation(
      testSuite.rpc,
      delegationPda,
    );
    expect(delegationAccount.data.amountPulledInPeriod).toBe(transferAmount);
  });
});
