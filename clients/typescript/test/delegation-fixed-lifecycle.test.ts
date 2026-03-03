import { describe, expect, test } from 'vitest';
import {
  fetchFixedDelegation,
  fetchMaybeMultiDelegate,
  fetchMultiDelegate,
} from '../src/generated/index.ts';
import {
  buildCloseMultiDelegate,
  buildCreateFixedDelegation,
  buildInitMultiDelegate,
  buildRevokeDelegation,
} from '../src/instructions/delegation.ts';
import { getDelegationPDA, getMultiDelegatePDA } from '../src/pdas.ts';
import { addressAsSigner } from '../src/wallet.ts';
import {
  DEFAULT_TEST_BALANCE,
  getWalletProviders,
  initTestSuite,
  ONE_HOUR_IN_SECONDS,
} from './setup.ts';

describe.each(getWalletProviders())('Fixed Delegation Lifecycle ($name)', ({
  createWallet,
}) => {
  test('init → create → transfer → revoke → close', async () => {
    const t = await initTestSuite();
    const wallet = await createWallet(t);

    const userAta = await t.createAtaWithBalance(
      t.tokenMint,
      wallet.address,
      DEFAULT_TEST_BALANCE,
    );

    // 1. Init multi-delegate
    const { instructions: initIxs } = await buildInitMultiDelegate({
      owner: addressAsSigner(wallet.address),
      tokenMint: t.tokenMint,
      userAta,
      tokenProgram: t.tokenProgram,
    });
    await wallet.sendInstructions(initIxs);

    const [multiDelegatePda] = await getMultiDelegatePDA(
      wallet.address,
      t.tokenMint,
    );

    const multiDelegateAccount = await fetchMultiDelegate(
      t.rpc,
      multiDelegatePda,
    );
    expect(multiDelegateAccount.data.user).toBe(wallet.address);
    expect(multiDelegateAccount.data.tokenMint).toBe(t.tokenMint);

    // 2. Create fixed delegation
    const delegatee = await t.createFundedKeypair();
    const nonce = 0n;
    const amount = 500_000n;
    const currentTs = await t.getValidatorTime();
    const expiryS = currentTs + BigInt(ONE_HOUR_IN_SECONDS);

    const { instructions: createIxs } = await buildCreateFixedDelegation({
      delegator: addressAsSigner(wallet.address),
      tokenMint: t.tokenMint,
      delegatee: delegatee.address,
      nonce,
      amount,
      expiryTs: expiryS,
    });
    await wallet.sendInstructions(createIxs);

    const [delegationPda] = await getDelegationPDA(
      multiDelegatePda,
      wallet.address,
      delegatee.address,
      nonce,
    );

    const delegationAccount = await fetchFixedDelegation(t.rpc, delegationPda);
    expect(delegationAccount.data.amount).toBe(amount);
    expect(delegationAccount.data.expiryTs).toBe(expiryS);

    // 3. Transfer 100k (delegatee signs, not the wallet)
    const delegateeAta = await t.createAtaWithBalance(
      t.tokenMint,
      delegatee.address,
      0n,
    );

    const transferAmount = 100_000n;
    await t.client.transferFixed({
      delegatee,
      delegator: wallet.address,
      delegatorAta: userAta,
      tokenMint: t.tokenMint,
      delegationPda,
      amount: transferAmount,
      receiverAta: delegateeAta,
      tokenProgram: t.tokenProgram,
    });

    const balance = await t.rpc.getTokenAccountBalance(delegateeAta).send();
    expect(balance.value.amount).toBe(transferAmount.toString());

    const delegationAfterTransfer = await fetchFixedDelegation(
      t.rpc,
      delegationPda,
    );
    expect(delegationAfterTransfer.data.amount).toBe(amount - transferAmount);

    // 4. Revoke delegation
    const { instructions: revokeIxs } = buildRevokeDelegation({
      authority: addressAsSigner(wallet.address),
      delegationAccount: delegationPda,
    });
    await wallet.sendInstructions(revokeIxs);
    await expect(fetchFixedDelegation(t.rpc, delegationPda)).rejects.toThrow();

    // 5. Close multi-delegate
    const balanceBefore = await t.rpc.getBalance(wallet.address).send();

    const { instructions: closeIxs } = await buildCloseMultiDelegate({
      user: addressAsSigner(wallet.address),
      tokenMint: t.tokenMint,
    });
    await wallet.sendInstructions(closeIxs);

    const accountAfter = await fetchMaybeMultiDelegate(t.rpc, multiDelegatePda);
    expect(accountAfter.exists).toBe(false);

    const balanceAfter = await t.rpc.getBalance(wallet.address).send();
    expect(balanceAfter.value).toBeGreaterThan(balanceBefore.value);
  });
});
