import { describe, expect, test } from 'bun:test';
import { generateKeyPairSigner } from 'gill';
import {
  fetchFixedDelegation,
  fetchMultiDelegate,
  fetchRecurringDelegation,
} from '../src/generated/index.ts';
import { getDelegationPDA, getMultiDelegatePDA } from '../src/pdas.ts';
import { initTestSuite } from './setup.ts';

describe('MultiDelegator Integration Tests', () => {
  test('can connect to surfpool validator', async () => {
    const testSuite = await initTestSuite();

    const res = await testSuite.rpc.getHealth().send();

    expect(res).toBe('ok');
  });

  test('initialize multi delegate', async () => {
    const testSuite = await initTestSuite();

    const userAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      testSuite.payer.address,
      1_000_000n,
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

  test('create fixed delegation', async () => {
    const testSuite = await initTestSuite();

    const userAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      testSuite.payer.address,
      1_000_000n,
    );

    await testSuite.client.initMultiDelegate(
      testSuite.payer,
      testSuite.tokenMint,
      userAta,
    );

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;
    const amount = 500_000n;
    const expiryS = BigInt(Math.floor(Date.now() / 1000) + 3600);

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

    const fixedDelegation = await fetchFixedDelegation(
      testSuite.rpc,
      delegationPda,
    );

    expect(fixedDelegation).toBeDefined();
    expect(fixedDelegation.data.header.delegatee).toBe(delegatee.address);
    expect(fixedDelegation.data.amount).toBe(amount);
    expect(fixedDelegation.data.expiryS).toBe(expiryS);
  });

  test('create recurring delegation', async () => {
    const testSuite = await initTestSuite();

    const userAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      testSuite.payer.address,
      1_000_000n,
    );

    await testSuite.client.initMultiDelegate(
      testSuite.payer,
      testSuite.tokenMint,
      userAta,
    );

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;
    const amountPerPeriod = 100_000n;
    const periodLengthS = 86400n;
    const expiryS = BigInt(Math.floor(Date.now() / 1000) + 86400 * 30);

    const result = await testSuite.client.createRecurringDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amountPerPeriod,
      periodLengthS,
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

    const recurringDelegation = await fetchRecurringDelegation(
      testSuite.rpc,
      delegationPda,
    );

    expect(recurringDelegation).toBeDefined();
    expect(recurringDelegation.data.header.delegatee).toBe(delegatee.address);
    expect(recurringDelegation.data.amountPerPeriod).toBe(amountPerPeriod);
    expect(recurringDelegation.data.periodLengthS).toBe(periodLengthS);
    expect(recurringDelegation.data.expiryS).toBe(expiryS);
  });
});
