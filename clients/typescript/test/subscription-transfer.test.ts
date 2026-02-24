import { describe, expect, test } from 'vitest';
import { fetchSubscriptionDelegation } from '../src/generated/index.ts';
import { DEFAULT_TEST_BALANCE, initTestSuite } from './setup.ts';

describe('TransferSubscription', () => {
  test('happy path - plan owner pulls', async () => {
    const t = await initTestSuite();
    const planAmount = 1_000_000n;

    // Merchant creates plan (payer is plan owner)
    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      planAmount,
      1n, // 1 hour period
      0n,
      [],
      [],
      'https://example.com/plan.json',
    );

    // Subscriber setup
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

    // Create receiver ATA for the plan owner
    const receiverAta = await t.createAtaWithBalance(
      t.tokenMint,
      t.payer.address,
      0n,
    );

    // Plan owner pulls tokens
    const transferAmount = 500_000n;
    const { signature } = await t.client.transferSubscription(
      t.payer,
      subscriber.address,
      t.tokenMint,
      subscriptionPda,
      planPda,
      transferAmount,
      receiverAta,
      t.tokenProgram,
    );

    expect(signature).toBeDefined();

    // Verify receiver got tokens
    const balance = await t.rpc.getTokenAccountBalance(receiverAta).send();
    expect(balance.value.amount).toBe(transferAmount.toString());

    // Verify subscription state updated
    const sub = (await fetchSubscriptionDelegation(t.rpc, subscriptionPda))
      .data;
    expect(sub.amountPulledInPeriod).toBe(transferAmount);
  });

  test('multiple pulls within period', async () => {
    const t = await initTestSuite();
    const planAmount = 1_000_000n;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      planAmount,
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

    const receiverAta = await t.createAtaWithBalance(
      t.tokenMint,
      t.payer.address,
      0n,
    );

    // First pull
    await t.client.transferSubscription(
      t.payer,
      subscriber.address,
      t.tokenMint,
      subscriptionPda,
      planPda,
      400_000n,
      receiverAta,
      t.tokenProgram,
    );

    // Second pull
    await t.client.transferSubscription(
      t.payer,
      subscriber.address,
      t.tokenMint,
      subscriptionPda,
      planPda,
      400_000n,
      receiverAta,
      t.tokenProgram,
    );

    // Verify total pulled
    const balance = await t.rpc.getTokenAccountBalance(receiverAta).send();
    expect(balance.value.amount).toBe('800000');

    const sub = (await fetchSubscriptionDelegation(t.rpc, subscriptionPda))
      .data;
    expect(sub.amountPulledInPeriod).toBe(800_000n);
  });

  test('exceeding period limit rejects', async () => {
    const t = await initTestSuite();
    const planAmount = 500_000n;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      planAmount,
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

    const receiverAta = await t.createAtaWithBalance(
      t.tokenMint,
      t.payer.address,
      0n,
    );

    // Try to pull more than the period limit
    await expect(
      t.client.transferSubscription(
        t.payer,
        subscriber.address,
        t.tokenMint,
        subscriptionPda,
        planPda,
        planAmount + 1n,
        receiverAta,
        t.tokenProgram,
      ),
    ).rejects.toThrow();
  });

  test('unauthorized caller rejects', async () => {
    const t = await initTestSuite();

    const { planPda } = await t.client.createPlan(
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

    // Random person tries to pull
    const randomCaller = await t.createFundedKeypair();
    const receiverAta = await t.createAtaWithBalance(
      t.tokenMint,
      randomCaller.address,
      0n,
    );

    await expect(
      t.client.transferSubscription(
        randomCaller,
        subscriber.address,
        t.tokenMint,
        subscriptionPda,
        planPda,
        100_000n,
        receiverAta,
        t.tokenProgram,
      ),
    ).rejects.toThrow();
  });

  test('whitelisted puller can pull', async () => {
    const t = await initTestSuite();
    const puller = await t.createFundedKeypair();

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      1n,
      0n,
      [],
      [puller.address],
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

    const receiverAta = await t.createAtaWithBalance(
      t.tokenMint,
      puller.address,
      0n,
    );

    const { signature } = await t.client.transferSubscription(
      puller,
      subscriber.address,
      t.tokenMint,
      subscriptionPda,
      planPda,
      100_000n,
      receiverAta,
      t.tokenProgram,
    );

    expect(signature).toBeDefined();

    const balance = await t.rpc.getTokenAccountBalance(receiverAta).send();
    expect(balance.value.amount).toBe('100000');
  });
});
