import type {
  Address,
  GetLatestBlockhashApi,
  Instruction,
  Rpc,
  TransactionSigner,
} from 'gill';
import { createTransaction, signTransactionMessageWithSigners } from 'gill';
import {
  getCreateFixedDelegationInstruction,
  getCreateRecurringDelegationInstruction,
  getInitMultiDelegateInstruction,
} from './generated/index.js';
import { getDelegationPDA, getMultiDelegatePDA } from './pdas.js';

type SolanaClient = {
  rpc: Rpc<GetLatestBlockhashApi>;
  sendAndConfirmTransaction: (
    tx: Awaited<ReturnType<typeof signTransactionMessageWithSigners>>,
  ) => Promise<string>;
};

export class MultiDelegatorClient {
  constructor(public readonly client: SolanaClient) {}

  private async buildAndSendTransaction(
    instructions: Instruction[],
    signers: TransactionSigner[],
  ): Promise<string> {
    const { value: latestBlockhash } = await this.client.rpc
      .getLatestBlockhash()
      .send();

    const transactionMessage = createTransaction({
      instructions,
      feePayer: signers[0],
      latestBlockhash,
    });
    const signedTransaction =
      await signTransactionMessageWithSigners(transactionMessage);
    const signature =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    return signature;
  }

  async initMultiDelegate(
    owner: TransactionSigner,
    tokenMint: Address,
    userAta: Address,
  ): Promise<{ signature: string }> {
    const user = owner.address;
    const [multiDelegate] = await getMultiDelegatePDA(user, tokenMint);

    const instruction = getInitMultiDelegateInstruction({
      owner,
      multiDelegate,
      tokenMint,
      userAta,
    });

    const sig = await this.buildAndSendTransaction([instruction], [owner]);
    return { signature: sig };
  }

  async createFixedDelegation(
    delegator: TransactionSigner,
    tokenMint: Address,
    delegatee: Address,
    nonce: number | bigint,
    amount: number | bigint,
    expiryS: number | bigint,
  ): Promise<{ signature: string }> {
    const user = delegator.address;
    const [multiDelegate] = await getMultiDelegatePDA(user, tokenMint);
    const [delegationAccount] = await getDelegationPDA(
      multiDelegate,
      user,
      delegatee,
      nonce,
    );

    const instruction = getCreateFixedDelegationInstruction({
      delegator,
      multiDelegate,
      delegationAccount,
      delegatee,
      nonce,
      amount,
      expiryS,
    });

    const sig = await this.buildAndSendTransaction([instruction], [delegator]);
    return { signature: sig };
  }

  async createRecurringDelegation(
    delegator: TransactionSigner,
    tokenMint: Address,
    delegatee: Address,
    nonce: number | bigint,
    amountPerPeriod: number | bigint,
    periodLengthS: number | bigint,
    expiryS: number | bigint,
  ): Promise<{ signature: string }> {
    const user = delegator.address;
    const [multiDelegate] = await getMultiDelegatePDA(user, tokenMint);
    const [delegationAccount] = await getDelegationPDA(
      multiDelegate,
      user,
      delegatee,
      nonce,
    );

    const instruction = getCreateRecurringDelegationInstruction({
      delegator,
      multiDelegate,
      delegationAccount,
      delegatee,
      nonce,
      amountPerPeriod,
      periodLengthS,
      expiryS,
    });

    const sig = await this.buildAndSendTransaction([instruction], [delegator]);
    return { signature: sig };
  }
}
