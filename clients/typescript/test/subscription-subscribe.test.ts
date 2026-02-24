import { describe, expect, test } from 'vitest';
import { fetchSubscriptionDelegation } from '../src/generated/index.ts';
import { getSubscriptionPDA } from '../src/pdas.ts';
import {
  DEFAULT_TEST_BALANCE,
  initTestSuite,
  ONE_HOUR_IN_SECONDS,
} from './setup.ts';

describe('Subscribe', () => {
  test('happy path', async () => {
    const t = await initTestSuite();

    // Create plan as merchant
    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      BigInt(ONE_HOUR_IN_SECONDS / 3600),
      0n,
      [],
      [],
      'https://example.com/plan.json',
    );

    // Subscriber setup: fund ATA + init multi-delegate
    const subscriber = await t.createFundedKeypair();
    const subscriberAta = await t.createAtaWithBalance(
      t.tokenMint,
      subscriber.address,
      DEFAULT_TEST_BALANCE,
    );
    await t.client.initMultiDelegate(
      subscriber,
      t.tokenMint,
      subscriberAta,
      t.tokenProgram,
    );

    // Subscribe
    const { signature, subscriptionPda } = await t.client.subscribe(
      subscriber,
      t.payer.address,
      1n,
      t.tokenMint,
    );

    expect(signature).toBeDefined();

    // Verify PDA matches expected
    const [expectedPda] = await getSubscriptionPDA(planPda, subscriber.address);
    expect(subscriptionPda).toBe(expectedPda);

    // Fetch and verify on-chain state
    const sub = (await fetchSubscriptionDelegation(t.rpc, subscriptionPda))
      .data;
    expect(sub.header.delegator).toBe(subscriber.address);
    expect(sub.amountPulledInPeriod).toBe(0n);
    expect(sub.expiresAtTs).toBe(0n);
  });

  test('subscribe without multi-delegate rejects', async () => {
    const t = await initTestSuite();

    await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      1n,
      0n,
      [],
      [],
      'https://example.com/plan.json',
    );

    // Subscriber has no multi-delegate initialized
    const subscriber = await t.createFundedKeypair();

    await expect(
      t.client.subscribe(subscriber, t.payer.address, 1n, t.tokenMint),
    ).rejects.toThrow();
  });

  test('double subscribe rejects', async () => {
    const t = await initTestSuite();

    await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      1n,
      0n,
      [],
      [],
      'https://example.com/plan.json',
    );

    const subscriber = await t.createFundedKeypair();
    const subscriberAta = await t.createAtaWithBalance(
      t.tokenMint,
      subscriber.address,
      DEFAULT_TEST_BALANCE,
    );
    await t.client.initMultiDelegate(
      subscriber,
      t.tokenMint,
      subscriberAta,
      t.tokenProgram,
    );

    await t.client.subscribe(subscriber, t.payer.address, 1n, t.tokenMint);

    // Second subscribe should fail
    await expect(
      t.client.subscribe(subscriber, t.payer.address, 1n, t.tokenMint),
    ).rejects.toThrow();
  });
});
