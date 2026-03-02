import {
  type createSolanaClient,
  createTransaction,
  generateKeyPairSigner,
  lamports,
  signTransactionMessageWithSigners,
} from 'gill';
import { describe, expect, test } from 'vitest';
import { ZERO_ADDRESS } from '../src/constants.ts';
import {
  fetchMaybePlan,
  getUpdatePlanInstruction,
  PlanStatus,
} from '../src/generated/index.ts';
import { initTestSuite } from './setup.ts';

type SolanaRpc = ReturnType<typeof createSolanaClient>['rpc'];

async function getValidatorTime(rpc: SolanaRpc): Promise<bigint> {
  const slot = await rpc.getSlot().send();
  const blockTime = await rpc.getBlockTime(slot).send();
  if (blockTime == null) throw new Error('blockTime is null');
  return BigInt(blockTime);
}

async function advanceClock(rpc: SolanaRpc, seconds: number): Promise<void> {
  const start = await getValidatorTime(rpc);
  const target = start + BigInt(seconds);
  while ((await getValidatorTime(rpc)) <= target) {
    const kp = await generateKeyPairSigner();
    await rpc.requestAirdrop(kp.address, lamports(1_000n)).send();
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
}

describe('DeletePlan', () => {
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
      '',
    );

    const validatorTs = await getValidatorTime(t.rpc);
    const endTs = validatorTs + 10n;

    const updateIx = getUpdatePlanInstruction({
      owner: t.payer,
      planPda,
      updatePlanData: {
        status: PlanStatus.Sunset,
        endTs,
        pullers: [ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS],
        metadataUri: '',
      },
    });
    const { value: latestBlockhash } = await t.rpc.getLatestBlockhash().send();
    const txMsg = createTransaction({
      instructions: [updateIx],
      feePayer: t.payer,
      latestBlockhash,
    });
    const signed = await signTransactionMessageWithSigners(txMsg);
    await t.client.client.sendAndConfirmTransaction(signed);

    await advanceClock(t.rpc, 12);

    const { signature } = await t.client.deletePlan(t.payer, planPda);
    expect(signature).toBeDefined();

    const account = await fetchMaybePlan(t.rpc, planPda);
    expect(account.exists).toBe(false);
  });

  test('non-owner rejected', async () => {
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

    const validatorTs = await getValidatorTime(t.rpc);
    const endTs = validatorTs + 10n;

    const updateIx = getUpdatePlanInstruction({
      owner: t.payer,
      planPda,
      updatePlanData: {
        status: PlanStatus.Sunset,
        endTs,
        pullers: [ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS],
        metadataUri: '',
      },
    });
    const { value: latestBlockhash } = await t.rpc.getLatestBlockhash().send();
    const txMsg = createTransaction({
      instructions: [updateIx],
      feePayer: t.payer,
      latestBlockhash,
    });
    const signed = await signTransactionMessageWithSigners(txMsg);
    await t.client.client.sendAndConfirmTransaction(signed);

    await advanceClock(t.rpc, 12);

    const nonOwner = await t.createFundedKeypair();
    await expect(t.client.deletePlan(nonOwner, planPda)).rejects.toThrow();
  });

  test('active expired plan deleted', async () => {
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

    const validatorTs = await getValidatorTime(t.rpc);
    const endTs = validatorTs + 10n;

    const updateIx = getUpdatePlanInstruction({
      owner: t.payer,
      planPda,
      updatePlanData: {
        status: PlanStatus.Active,
        endTs,
        pullers: [ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS],
        metadataUri: '',
      },
    });
    const { value: latestBlockhash } = await t.rpc.getLatestBlockhash().send();
    const txMsg = createTransaction({
      instructions: [updateIx],
      feePayer: t.payer,
      latestBlockhash,
    });
    const signed = await signTransactionMessageWithSigners(txMsg);
    await t.client.client.sendAndConfirmTransaction(signed);

    await advanceClock(t.rpc, 12);

    const { signature } = await t.client.deletePlan(t.payer, planPda);
    expect(signature).toBeDefined();

    const account = await fetchMaybePlan(t.rpc, planPda);
    expect(account.exists).toBe(false);
  });

  test('perpetual plan rejected', async () => {
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

    await expect(t.client.deletePlan(t.payer, planPda)).rejects.toThrow();
  });

  test('sunset not expired rejected', async () => {
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

    const endTs = BigInt(Math.floor(Date.now() / 1000) + 86400 * 30);
    await t.client.updatePlan(t.payer, planPda, PlanStatus.Sunset, endTs, '');

    await expect(t.client.deletePlan(t.payer, planPda)).rejects.toThrow();
  });
});
