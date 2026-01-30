import { describe, expect, test } from 'bun:test';
import { generateKeyPairSigner } from 'gill';
import type { Delegation } from '../src/client.ts';
import {
  fetchFixedDelegation,
  fetchMultiDelegate,
  fetchRecurringDelegation,
} from '../src/generated/index.ts';
import { getDelegationPDA, getMultiDelegatePDA } from '../src/pdas.ts';
import { initTestSuite } from './setup.ts';

const DEFAULT_TEST_BALANCE = 1_000_000n;
const ONE_HOUR_IN_SECONDS = 3600;
const ONE_DAY_IN_SECONDS = 86400;

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

    await expect(
      fetchRecurringDelegation(testSuite.rpc, delegationPda),
    ).rejects.toThrow();
  });

  test('get delegations for wallet', async () => {
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

    const delegatee1 = await generateKeyPairSigner();
    const delegatee2 = await generateKeyPairSigner();

    await testSuite.client.createFixedDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee1.address,
      0n,
      100_000n,
      BigInt(Math.floor(Date.now() / 1000) + ONE_HOUR_IN_SECONDS),
    );

    await testSuite.client.createRecurringDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee2.address,
      1n,
      50_000n,
      BigInt(ONE_DAY_IN_SECONDS),
      BigInt(Math.floor(Date.now() / 1000) + ONE_DAY_IN_SECONDS * 30),
    );

    const delegations = await testSuite.client.getDelegationsForWallet(
      testSuite.payer.address,
    );

    expect(delegations.length).toBe(2);

    const fixedDelegations = delegations.filter(
      (d): d is Delegation & { kind: 'fixed' } => d.kind === 'fixed',
    );
    const recurringDelegations = delegations.filter(
      (d): d is Delegation & { kind: 'recurring' } => d.kind === 'recurring',
    );

    expect(fixedDelegations.length).toBe(1);
    expect(recurringDelegations.length).toBe(1);

    expect(fixedDelegations[0].data.header.delegatee).toBe(delegatee1.address);
    expect(fixedDelegations[0].data.amount).toBe(100_000n);

    expect(recurringDelegations[0].data.header.delegatee).toBe(
      delegatee2.address,
    );
    expect(recurringDelegations[0].data.amountPerPeriod).toBe(50_000n);
  });
});
