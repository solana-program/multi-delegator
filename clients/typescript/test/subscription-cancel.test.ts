import { describe, expect, test } from 'vitest';
import { fetchSubscriptionDelegation } from '../src/generated/index.ts';
import { getPlanPDA } from '../src/pdas.ts';
import { DEFAULT_TEST_BALANCE, initTestSuite } from './setup.ts';

describe('CancelSubscription', () => {
  test('happy path', async () => {
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

    const { subscriptionPda } = await t.client.subscribe(
      subscriber,
      t.payer.address,
      1n,
      t.tokenMint,
    );

    const subBefore = (
      await fetchSubscriptionDelegation(t.rpc, subscriptionPda)
    ).data;
    expect(subBefore.expiresAtTs).toBe(0n);

    const [planPda] = await getPlanPDA(t.payer.address, 1n);
    const { signature } = await t.client.cancelSubscription(
      subscriber,
      planPda,
      subscriptionPda,
    );
    expect(signature).toBeDefined();

    const subAfter = (await fetchSubscriptionDelegation(t.rpc, subscriptionPda))
      .data;
    expect(subAfter.expiresAtTs).not.toBe(0n);
  });

  test('non-subscriber cannot cancel', async () => {
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

    const { subscriptionPda } = await t.client.subscribe(
      subscriber,
      t.payer.address,
      1n,
      t.tokenMint,
    );

    const [planPda] = await getPlanPDA(t.payer.address, 1n);
    const impostor = await t.createFundedKeypair();
    await expect(
      t.client.cancelSubscription(impostor, planPda, subscriptionPda),
    ).rejects.toThrow();
  });

  test('double cancel rejects', async () => {
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

    const { subscriptionPda } = await t.client.subscribe(
      subscriber,
      t.payer.address,
      1n,
      t.tokenMint,
    );

    const [planPda] = await getPlanPDA(t.payer.address, 1n);
    await t.client.cancelSubscription(subscriber, planPda, subscriptionPda);

    await expect(
      t.client.cancelSubscription(subscriber, planPda, subscriptionPda),
    ).rejects.toThrow();
  });
});
