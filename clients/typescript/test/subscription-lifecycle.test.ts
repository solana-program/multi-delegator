import { describe, expect, test } from 'vitest';
import {
  fetchMaybePlan,
  fetchSubscriptionDelegation,
  PlanStatus,
} from '../src/generated/index.ts';
import { DEFAULT_TEST_BALANCE, initTestSuite } from './setup.ts';

describe('Subscription Lifecycle', () => {
  test('full lifecycle: create, subscribe, pull, cancel, sunset, delete', async () => {
    const t = await initTestSuite();
    const planAmount = 500_000n;

    // 1. Merchant creates a plan
    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      planAmount,
      1n, // 1 hour period
      0n, // perpetual
      [],
      [],
      'https://example.com/plan.json',
    );

    // 2. Subscriber sets up and subscribes
    const subscriber = await t.createFundedWallet();
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

    // Verify subscription state
    const subAfterSubscribe = (
      await fetchSubscriptionDelegation(t.rpc, subscriptionPda)
    ).data;
    expect(subAfterSubscribe.header.delegator).toBe(subscriber.address);
    expect(subAfterSubscribe.amountPulledInPeriod).toBe(0n);
    expect(subAfterSubscribe.expiresAtTs).toBe(0n);

    // 3. Merchant pulls funds
    const merchantAta = await t.createAtaWithBalance(
      t.tokenMint,
      t.payer.address,
      0n,
    );

    const pullAmount = 200_000n;
    await t.client.transferSubscription(
      t.payerKeypair,
      subscriber.address,
      t.tokenMint,
      subscriptionPda,
      planPda,
      pullAmount,
      merchantAta,
      t.tokenProgram,
    );

    const merchantBalance = await t.rpc
      .getTokenAccountBalance(merchantAta)
      .send();
    expect(merchantBalance.value.amount).toBe(pullAmount.toString());

    const subAfterPull = (
      await fetchSubscriptionDelegation(t.rpc, subscriptionPda)
    ).data;
    expect(subAfterPull.amountPulledInPeriod).toBe(pullAmount);

    // 4. Subscriber cancels
    await t.client.cancelSubscription(subscriber, planPda, subscriptionPda);

    const subAfterCancel = (
      await fetchSubscriptionDelegation(t.rpc, subscriptionPda)
    ).data;
    expect(subAfterCancel.expiresAtTs).not.toBe(0n);

    // 5. Merchant sunsets the plan
    const validatorTs = await t.getValidatorTime();
    const endTs = validatorTs + 10n;

    await t.client.updatePlan(
      t.payer,
      planPda,
      PlanStatus.Sunset,
      endTs,
      'https://example.com/plan.json',
    );

    // 6. Time-travel past endTs, then delete the plan
    await t.timeTravel(Number(endTs) + 5);

    const { signature: deleteSig } = await t.client.deletePlan(
      t.payer,
      planPda,
    );
    expect(deleteSig).toBeDefined();

    const planAfterDelete = await fetchMaybePlan(t.rpc, planPda);
    expect(planAfterDelete.exists).toBe(false);
  });

  test('whitelisted puller can transfer', async () => {
    const t = await initTestSuite();
    const puller = await t.createFundedKeypair();

    // 1. Merchant creates plan with a whitelisted puller
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

    // 2. Subscriber subscribes
    const subscriber = await t.createFundedWallet();
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

    // 3. Puller (not the merchant) pulls funds
    const pullerAta = await t.createAtaWithBalance(
      t.tokenMint,
      puller.address,
      0n,
    );

    const pullAmount = 100_000n;
    const { signature } = await t.client.transferSubscription(
      puller,
      subscriber.address,
      t.tokenMint,
      subscriptionPda,
      planPda,
      pullAmount,
      pullerAta,
      t.tokenProgram,
    );

    expect(signature).toBeDefined();

    const balance = await t.rpc.getTokenAccountBalance(pullerAta).send();
    expect(balance.value.amount).toBe(pullAmount.toString());
  });
});
