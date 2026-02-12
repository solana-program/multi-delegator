import { describe, expect, test } from 'bun:test';
import type { Address, TransactionSigner } from 'gill';
import { generateKeyPairSigner } from 'gill';
import {
  fetchFixedDelegation,
  fetchRecurringDelegation,
  getCreateFixedDelegationInstruction,
  getCreateRecurringDelegationInstruction,
  getInitMultiDelegateInstruction,
  getRevokeDelegationInstruction,
} from '../src/generated/index.ts';
import { getDelegationPDA, getMultiDelegatePDA } from '../src/pdas.ts';
import {
  DEFAULT_TEST_BALANCE,
  getSmartWallet,
  getSmartWalletList,
  initTestSuite,
  ONE_DAY_IN_SECONDS,
  ONE_HOUR_IN_SECONDS,
  type SmartWalletName,
} from './setup.ts';
import type { SmartWallet } from './smart-wallets/index.ts';

const asSigner = (address: Address) =>
  ({ address }) as unknown as TransactionSigner;

async function initMultiDelegateWithWallet(
  wallet: SmartWallet,
  tokenMint: Address,
  userAta: Address,
) {
  const [multiDelegate] = await getMultiDelegatePDA(wallet.address, tokenMint);
  const instruction = getInitMultiDelegateInstruction({
    owner: asSigner(wallet.address),
    multiDelegate,
    tokenMint,
    userAta,
  });
  await wallet.sendInstructions([instruction]);
  return multiDelegate;
}

async function createFixedDelegationWithWallet(
  wallet: SmartWallet,
  tokenMint: Address,
  delegatee: Address,
  nonce: bigint,
  amount: bigint,
  expiryTs: bigint,
) {
  const [multiDelegate] = await getMultiDelegatePDA(wallet.address, tokenMint);
  const [delegationAccount] = await getDelegationPDA(
    multiDelegate,
    wallet.address,
    delegatee,
    nonce,
  );
  const instruction = getCreateFixedDelegationInstruction({
    delegator: asSigner(wallet.address),
    multiDelegate,
    delegationAccount,
    delegatee,
    nonce,
    amount,
    expiryTs,
  });
  await wallet.sendInstructions([instruction]);
  return { multiDelegate, delegationAccount };
}

async function createRecurringDelegationWithWallet(
  wallet: SmartWallet,
  tokenMint: Address,
  delegatee: Address,
  nonce: bigint,
  amountPerPeriod: bigint,
  periodLengthS: bigint,
  startTs: bigint,
  expiryTs: bigint,
) {
  const [multiDelegate] = await getMultiDelegatePDA(wallet.address, tokenMint);
  const [delegationAccount] = await getDelegationPDA(
    multiDelegate,
    wallet.address,
    delegatee,
    nonce,
  );
  const instruction = getCreateRecurringDelegationInstruction({
    delegator: asSigner(wallet.address),
    multiDelegate,
    delegationAccount,
    delegatee,
    nonce,
    amountPerPeriod,
    periodLengthS,
    startTs,
    expiryTs,
  });
  await wallet.sendInstructions([instruction]);
  return { multiDelegate, delegationAccount };
}

async function revokeDelegationWithWallet(
  wallet: SmartWallet,
  delegationAccount: Address,
) {
  const instruction = getRevokeDelegationInstruction({
    authority: asSigner(wallet.address),
    delegationAccount,
  });
  await wallet.sendInstructions([instruction]);
}

async function initWalletContext(walletName: SmartWalletName) {
  const testSuite = await initTestSuite();
  const wallet = await getSmartWallet(testSuite, walletName);
  const userAta = await testSuite.createAtaWithBalance(
    testSuite.tokenMint,
    wallet.address,
    DEFAULT_TEST_BALANCE,
  );
  await initMultiDelegateWithWallet(wallet, testSuite.tokenMint, userAta);
  return { testSuite, wallet, userAta };
}

const wallets = getSmartWalletList().map((name) => ({ name }));

describe.each(wallets)('wallet $name tests', ({ name }) => {
  test('create fixed delegation', async () => {
    const { testSuite, wallet } = await initWalletContext(name);

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;
    const amount = 500_000n;
    const expiryTs = BigInt(
      Math.floor(Date.now() / 1000) + ONE_HOUR_IN_SECONDS,
    );

    const { delegationAccount } = await createFixedDelegationWithWallet(
      wallet,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amount,
      expiryTs,
    );

    const delegation = await fetchFixedDelegation(
      testSuite.rpc,
      delegationAccount,
    );
    expect(delegation.data.amount).toBe(amount);
    expect(delegation.data.expiryTs).toBe(expiryTs);
  }, 15000);

  test('revoke fixed delegation', async () => {
    const { testSuite, wallet } = await initWalletContext(name);

    const delegatee = await generateKeyPairSigner();
    const { delegationAccount } = await createFixedDelegationWithWallet(
      wallet,
      testSuite.tokenMint,
      delegatee.address,
      0n,
      500_000n,
      BigInt(Math.floor(Date.now() / 1000) + ONE_HOUR_IN_SECONDS),
    );

    const delegatorBalanceBefore = await testSuite.rpc
      .getBalance(wallet.address)
      .send();

    await revokeDelegationWithWallet(wallet, delegationAccount);

    expect(
      fetchFixedDelegation(testSuite.rpc, delegationAccount),
    ).rejects.toThrow();

    const delegatorBalanceAfter = await testSuite.rpc
      .getBalance(wallet.address)
      .send();
    expect(delegatorBalanceAfter.value).toBeGreaterThan(
      delegatorBalanceBefore.value,
    );
  }, 15000);

  test('transfer fixed delegation', async () => {
    const { testSuite, wallet, userAta } = await initWalletContext(name);

    const delegatee = await testSuite.createFundedKeypair();
    const nonce = 0n;
    const amount = 500_000n;
    const expiryTs = BigInt(
      Math.floor(Date.now() / 1000) + ONE_HOUR_IN_SECONDS,
    );

    const { delegationAccount } = await createFixedDelegationWithWallet(
      wallet,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amount,
      expiryTs,
    );

    const delegateeAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      delegatee.address,
      0n,
    );

    const transferAmount = 100_000n;
    const result = await testSuite.client.transferFixed(
      delegatee,
      wallet.address,
      userAta,
      testSuite.tokenMint,
      delegationAccount,
      transferAmount,
      delegateeAta,
    );

    expect(result.signature).toBeDefined();

    const balance = await testSuite.rpc
      .getTokenAccountBalance(delegateeAta)
      .send();
    expect(balance.value.amount).toBe(transferAmount.toString());

    const delegationAccountInfo = await fetchFixedDelegation(
      testSuite.rpc,
      delegationAccount,
    );
    expect(delegationAccountInfo.data.amount).toBe(amount - transferAmount);
  }, 15000);

  test('create recurring delegation', async () => {
    const { testSuite, wallet } = await initWalletContext(name);

    const delegatee = await generateKeyPairSigner();
    const nonce = 0n;
    const amountPerPeriod = 100_000n;
    const periodLengthS = BigInt(ONE_DAY_IN_SECONDS);
    const startTs = BigInt(Math.floor(Date.now() / 1000));
    const expiryTs = BigInt(
      Math.floor(Date.now() / 1000) + ONE_DAY_IN_SECONDS * 30,
    );

    const { delegationAccount } = await createRecurringDelegationWithWallet(
      wallet,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amountPerPeriod,
      periodLengthS,
      startTs,
      expiryTs,
    );

    const delegation = await fetchRecurringDelegation(
      testSuite.rpc,
      delegationAccount,
    );
    expect(delegation.data.expiryTs).toBe(expiryTs);
    expect(delegation.data.periodLengthS).toBe(periodLengthS);
    expect(delegation.data.currentPeriodStartTs).toBe(startTs);
    expect(delegation.data.amountPerPeriod).toBe(amountPerPeriod);
    expect(delegation.data.amountPulledInPeriod).toBe(0n);
  }, 15000);

  test('revoke recurring delegation', async () => {
    const { testSuite, wallet } = await initWalletContext(name);

    const delegatee = await generateKeyPairSigner();
    const { delegationAccount } = await createRecurringDelegationWithWallet(
      wallet,
      testSuite.tokenMint,
      delegatee.address,
      0n,
      100_000n,
      BigInt(ONE_DAY_IN_SECONDS),
      BigInt(Math.floor(Date.now() / 1000)),
      BigInt(Math.floor(Date.now() / 1000) + ONE_DAY_IN_SECONDS * 30),
    );

    await revokeDelegationWithWallet(wallet, delegationAccount);

    expect(
      fetchRecurringDelegation(testSuite.rpc, delegationAccount),
    ).rejects.toThrow();
  }, 15000);

  test('transfer recurring delegation', async () => {
    const { testSuite, wallet, userAta } = await initWalletContext(name);

    const delegatee = await testSuite.createFundedKeypair();
    const nonce = 0n;
    const amountPerPeriod = 100_000n;
    const periodLengthS = BigInt(ONE_DAY_IN_SECONDS);
    const startTs = BigInt(Math.floor(Date.now() / 1000));
    const expiryTs = BigInt(
      Math.floor(Date.now() / 1000) + ONE_DAY_IN_SECONDS * 30,
    );

    const { delegationAccount } = await createRecurringDelegationWithWallet(
      wallet,
      testSuite.tokenMint,
      delegatee.address,
      nonce,
      amountPerPeriod,
      periodLengthS,
      startTs,
      expiryTs,
    );

    const delegateeAta = await testSuite.createAtaWithBalance(
      testSuite.tokenMint,
      delegatee.address,
      0n,
    );

    const transferAmount = 50_000n;
    const result = await testSuite.client.transferRecurring(
      delegatee,
      wallet.address,
      userAta,
      testSuite.tokenMint,
      delegationAccount,
      transferAmount,
      delegateeAta,
    );

    expect(result.signature).toBeDefined();

    const balance = await testSuite.rpc
      .getTokenAccountBalance(delegateeAta)
      .send();
    expect(balance.value.amount).toBe(transferAmount.toString());

    const delegationAccountInfo = await fetchRecurringDelegation(
      testSuite.rpc,
      delegationAccount,
    );
    expect(delegationAccountInfo.data.amountPulledInPeriod).toBe(
      transferAmount,
    );
  }, 15000);
});
