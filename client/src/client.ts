import type { Address, Instruction, TransactionSigner } from 'gill';
import {
  type createSolanaClient,
  createTransaction,
  signTransactionMessageWithSigners,
} from 'gill';
import {
  getCreateSimpleDelegationInstruction,
  getInitMultiDelegateInstruction,
} from './generated/index.js';
import { getFixedDelegatePDA, getMultiDelegatePDA } from './pdas.js';

type SolanaClient = ReturnType<typeof createSolanaClient>;

interface TransactionResult {
  signature: string;
}

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
  ): Promise<TransactionResult> {
    const user = owner.address || (owner as TransactionSigner).address;
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

  async createSimpleDelegation(
    owner: TransactionSigner,
    tokenMint: Address,
    delegate: Address,
    kind: number,
    amount: number | bigint,
    expiryS: number | bigint,
  ): Promise<TransactionResult> {
    const user = owner.address || (owner as TransactionSigner).address;
    const [multiDelegate] = await getMultiDelegatePDA(user, tokenMint);
    const [delegateAccount] = await getFixedDelegatePDA(
      multiDelegate,
      delegate,
      user,
      kind,
    );

    const instruction = getCreateSimpleDelegationInstruction({
      user: owner,
      multiDelegate,
      delegateAccount,
      delegate,
      kind,
      amount,
      expiryS,
    });

    const sig = await this.buildAndSendTransaction([instruction], [owner]);
    return { signature: sig };
  }
}
