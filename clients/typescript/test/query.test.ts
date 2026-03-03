import { generateKeyPairSigner } from 'gill';
import { describe, expect, test } from 'vitest';
import type { Delegation } from '../src/client.ts';
import {
  DEFAULT_TEST_BALANCE,
  initTestSuite,
  ONE_DAY_IN_SECONDS,
  ONE_HOUR_IN_SECONDS,
} from './setup.ts';

describe('MultiDelegator Query Tests', () => {
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
      testSuite.tokenProgram,
    );

    const delegatee1 = await generateKeyPairSigner();
    const delegatee2 = await generateKeyPairSigner();

    const currentTs = await testSuite.getValidatorTime();

    await testSuite.client.createFixedDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee1.address,
      0n,
      100_000n,
      currentTs + BigInt(ONE_HOUR_IN_SECONDS),
    );

    await testSuite.client.createRecurringDelegation(
      testSuite.payer,
      testSuite.tokenMint,
      delegatee2.address,
      1n,
      50_000n,
      BigInt(ONE_DAY_IN_SECONDS),
      currentTs,
      currentTs + BigInt(ONE_DAY_IN_SECONDS * 30),
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
