import { generateKeyPairSigner } from 'gill';
import { describe, expect, test } from 'vitest';
import { ZERO_ADDRESS } from '../src/constants.ts';
import { fetchPlan, PlanStatus } from '../src/generated/index.ts';
import { initTestSuite } from './setup.ts';

describe('UpdatePlan', () => {
  test('happy path', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      720n,
      0n,
      [dest],
      [],
      'https://example.com/plan.json',
    );

    const futureEndTs = BigInt(Math.floor(Date.now() / 1000) + 86400 * 60);
    const { signature } = await t.client.updatePlan(
      t.payer,
      planPda,
      PlanStatus.Sunset,
      futureEndTs,
      'https://example.com/updated.json',
    );
    expect(signature).toBeDefined();

    const plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.status).toBe(PlanStatus.Sunset);
    expect(plan.data.metadataUri).toBe('https://example.com/updated.json');
  });

  test('preserves terms', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;
    const puller = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      720n,
      0n,
      [dest],
      [puller],
      'https://example.com/plan.json',
    );

    const futureEndTs = BigInt(Math.floor(Date.now() / 1000) + 86400 * 60);
    await t.client.updatePlan(
      t.payer,
      planPda,
      PlanStatus.Sunset,
      futureEndTs,
      'https://example.com/v2.json',
      [puller],
    );

    const plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.data.mint).toBe(t.tokenMint);
    expect(plan.data.amount).toBe(1_000_000n);
    expect(plan.data.periodHours).toBe(720n);
    expect(plan.data.planId).toBe(1n);
    expect(plan.data.destinations[0]).toBe(dest);
    expect(plan.data.pullers[0]).toBe(puller);
  });

  test('not owner rejects', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      720n,
      0n,
      [dest],
      [],
      '',
    );

    const nonOwner = await t.createFundedKeypair();

    const futureEndTs = BigInt(Math.floor(Date.now() / 1000) + 86400 * 60);
    await expect(
      t.client.updatePlan(
        nonOwner,
        planPda,
        PlanStatus.Sunset,
        futureEndTs,
        '',
      ),
    ).rejects.toThrow();
  });

  test('sunset is terminal', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      720n,
      0n,
      [dest],
      [],
      '',
    );

    const futureEndTs = BigInt(Math.floor(Date.now() / 1000) + 86400 * 60);
    await t.client.updatePlan(
      t.payer,
      planPda,
      PlanStatus.Sunset,
      futureEndTs,
      '',
    );

    const plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.status).toBe(PlanStatus.Sunset);

    await expect(
      t.client.updatePlan(t.payer, planPda, PlanStatus.Active, 0n, ''),
    ).rejects.toThrow();
  });

  test('sunset requires end_ts', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      720n,
      0n,
      [dest],
      [],
      '',
    );

    await expect(
      t.client.updatePlan(t.payer, planPda, PlanStatus.Sunset, 0n, ''),
    ).rejects.toThrow();
  });

  test('clear end_ts', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;

    const futureTs = BigInt(Math.floor(Date.now() / 1000) + 86400 * 60);
    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      720n,
      futureTs,
      [dest],
      [],
      '',
    );

    let plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.data.endTs).toBe(futureTs);

    await t.client.updatePlan(t.payer, planPda, PlanStatus.Active, 0n, '');

    plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.data.endTs).toBe(0n);
  });

  test('update pullers', async () => {
    const t = await initTestSuite();
    const pullerA = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000_000n,
      720n,
      0n,
      [],
      [pullerA],
      '',
    );

    let plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.data.pullers[0]).toBe(pullerA);

    const pullerB = (await generateKeyPairSigner()).address;
    await t.client.updatePlan(t.payer, planPda, PlanStatus.Active, 0n, '', [
      pullerB,
    ]);

    plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.data.pullers[0]).toBe(pullerB);
    expect(plan.data.pullers[1]).toBe(ZERO_ADDRESS);
  });
});
