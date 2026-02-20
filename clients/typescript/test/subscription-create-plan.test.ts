import { generateKeyPairSigner } from 'gill';
import { describe, expect, test } from 'vitest';
import { ZERO_ADDRESS } from '../src/constants.ts';
import { fetchPlan, PlanStatus } from '../src/generated/index.ts';
import { getPlanPDA } from '../src/pdas.ts';
import { initTestSuite } from './setup.ts';

describe('CreatePlan', () => {
  test('happy path', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;
    const puller = (await generateKeyPairSigner()).address;

    const { signature, planPda } = await t.client.createPlan(
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

    expect(signature).toBeDefined();

    const [expectedPda] = await getPlanPDA(t.payer.address, 1n);
    expect(planPda).toBe(expectedPda);

    const plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.owner).toBe(t.payer.address);
    expect(plan.status).toBe(PlanStatus.Active);
    expect(plan.data.planId).toBe(1n);
    expect(plan.data.mint).toBe(t.tokenMint);
    expect(plan.data.amount).toBe(1_000_000n);
    expect(plan.data.periodHours).toBe(720n);
    expect(plan.data.endTs).toBe(0n);
    expect(plan.data.destinations[0]).toBe(dest);
    expect(plan.data.pullers[0]).toBe(puller);
  });

  test('multiple destinations and pullers', async () => {
    const t = await initTestSuite();
    const dests = await Promise.all(
      [1, 2, 3].map(async () => (await generateKeyPairSigner()).address),
    );
    const pullers = await Promise.all(
      [1, 2].map(async () => (await generateKeyPairSigner()).address),
    );

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      500_000n,
      24n,
      0n,
      dests,
      pullers,
      'https://example.com/multi.json',
    );

    const { data: planData } = (await fetchPlan(t.rpc, planPda)).data;
    expect(planData.destinations[0]).toBe(dests[0]);
    expect(planData.destinations[1]).toBe(dests[1]);
    expect(planData.destinations[2]).toBe(dests[2]);
    expect(planData.pullers[0]).toBe(pullers[0]);
    expect(planData.pullers[1]).toBe(pullers[1]);
  });

  test('query plans for owner', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;
    const puller = (await generateKeyPairSigner()).address;

    await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      100n,
      24n,
      0n,
      [dest],
      [puller],
      'https://example.com/plan1.json',
    );
    await t.client.createPlan(
      t.payer,
      2n,
      t.tokenMint,
      200n,
      48n,
      0n,
      [dest],
      [puller],
      'https://example.com/plan2.json',
    );

    const plans = await t.client.getPlansForOwner(t.payer.address);
    expect(plans.length).toBe(2);

    const plan1 = plans.find((p) => p.data.data.planId === 1n);
    expect(plan1).toBeDefined();
    expect(plan1?.data.data.amount).toBe(100n);
    expect(plan1?.data.data.periodHours).toBe(24n);
    expect(plan1?.data.data.mint).toBe(t.tokenMint);
    expect(plan1?.data.data.destinations[0]).toBe(dest);
    expect(plan1?.data.data.pullers[0]).toBe(puller);
    expect(plan1?.data.data.metadataUri).toBe('https://example.com/plan1.json');

    const plan2 = plans.find((p) => p.data.data.planId === 2n);
    expect(plan2).toBeDefined();
    expect(plan2?.data.data.amount).toBe(200n);
    expect(plan2?.data.data.periodHours).toBe(48n);
    expect(plan2?.data.data.mint).toBe(t.tokenMint);
    expect(plan2?.data.data.destinations[0]).toBe(dest);
    expect(plan2?.data.data.pullers[0]).toBe(puller);
    expect(plan2?.data.data.metadataUri).toBe('https://example.com/plan2.json');
  });

  test('zero amount rejects', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;
    const puller = (await generateKeyPairSigner()).address;

    await expect(
      t.client.createPlan(
        t.payer,
        1n,
        t.tokenMint,
        0n,
        24n,
        0n,
        [dest],
        [puller],
        'https://example.com/plan.json',
      ),
    ).rejects.toThrow();
  });

  test('no destinations succeeds', async () => {
    const t = await initTestSuite();
    const puller = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000n,
      24n,
      0n,
      [],
      [puller],
      'https://example.com/plan.json',
    );

    expect(planPda).toBeDefined();
    const plan = (await fetchPlan(t.rpc, planPda)).data;
    for (const dest of plan.data.destinations) {
      expect(dest).toBe(ZERO_ADDRESS);
    }
  });

  test('partial destinations pads with zeros', async () => {
    const t = await initTestSuite();
    const dest1 = (await generateKeyPairSigner()).address;
    const dest2 = (await generateKeyPairSigner()).address;

    const { planPda } = await t.client.createPlan(
      t.payer,
      1n,
      t.tokenMint,
      1_000n,
      24n,
      0n,
      [dest1, dest2],
      [],
      'https://example.com/plan.json',
    );

    const plan = (await fetchPlan(t.rpc, planPda)).data;
    expect(plan.data.destinations[0]).toBe(dest1);
    expect(plan.data.destinations[1]).toBe(dest2);
    expect(plan.data.destinations[2]).toBe(ZERO_ADDRESS);
    expect(plan.data.destinations[3]).toBe(ZERO_ADDRESS);
  });

  test('invalid period rejects', async () => {
    const t = await initTestSuite();
    const dest = (await generateKeyPairSigner()).address;
    const puller = (await generateKeyPairSigner()).address;

    await expect(
      t.client.createPlan(
        t.payer,
        1n,
        t.tokenMint,
        1_000n,
        0n,
        0n,
        [dest],
        [puller],
        'https://example.com/plan.json',
      ),
    ).rejects.toThrow();
  });
});
