import type {
  Address,
  GetAccountInfoApi,
  GetLatestBlockhashApi,
  GetProgramAccountsApi,
  Instruction,
  Rpc,
  TransactionSigner,
} from 'gill';
import {
  createTransaction,
  getBase64Encoder,
  signTransactionMessageWithSigners,
} from 'gill';
import { DELEGATOR_OFFSET, KIND_DISCRIMINATOR_OFFSET } from './constants.js';
import {
  DelegationKind,
  decodeFixedDelegation,
  decodeRecurringDelegation,
  type FixedDelegation,
  fetchMaybeMultiDelegate,
  getCreateFixedDelegationInstruction,
  getCreateRecurringDelegationInstruction,
  getInitMultiDelegateInstruction,
  getRevokeDelegationInstruction,
  getTransferFixedInstruction,
  getTransferRecurringInstruction,
  MULTI_DELEGATOR_PROGRAM_ADDRESS,
  type RecurringDelegation,
} from './generated/index.js';
import { getDelegationPDA, getMultiDelegatePDA } from './pdas.js';

type SolanaClient = {
  rpc: Rpc<GetAccountInfoApi & GetLatestBlockhashApi & GetProgramAccountsApi>;
  sendAndConfirmTransaction: (
    tx: Awaited<ReturnType<typeof signTransactionMessageWithSigners>>,
  ) => Promise<string>;
};

export type Delegation =
  | { kind: 'fixed'; address: Address; data: FixedDelegation }
  | { kind: 'recurring'; address: Address; data: RecurringDelegation };

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
    tokenProgram?: Address,
  ): Promise<{ signature: string }> {
    const user = owner.address;
    const [multiDelegate] = await getMultiDelegatePDA(user, tokenMint);

    const instruction = getInitMultiDelegateInstruction({
      owner,
      multiDelegate,
      tokenMint,
      userAta,
      tokenProgram,
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
    expiryTs: number | bigint,
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
      expiryTs,
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
    startTs: number | bigint,
    expiryTs: number | bigint,
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
      startTs,
      expiryTs,
    });

    const sig = await this.buildAndSendTransaction([instruction], [delegator]);
    return { signature: sig };
  }

  async revokeDelegation(
    delegator: TransactionSigner,
    delegationAccount: Address,
  ): Promise<{ signature: string }> {
    const instruction = getRevokeDelegationInstruction({
      authority: delegator,
      delegationAccount,
    });

    const sig = await this.buildAndSendTransaction([instruction], [delegator]);
    return { signature: sig };
  }

  private async transfer(
    kind: 'fixed' | 'recurring',
    delegatee: TransactionSigner,
    delegator: Address,
    delegatorAta: Address,
    tokenMint: Address,
    delegationPda: Address,
    amount: number | bigint,
    receiverAta: Address,
  ): Promise<{ signature: string }> {
    const [multiDelegate] = await getMultiDelegatePDA(delegator, tokenMint);

    const transferParams = {
      delegationPda,
      multiDelegate,
      delegatorAta,
      receiverAta,
      delegatee,
      transferData: {
        amount,
        delegator,
        mint: tokenMint,
      },
    };

    const instruction =
      kind === 'fixed'
        ? getTransferFixedInstruction(transferParams)
        : getTransferRecurringInstruction(transferParams);

    const sig = await this.buildAndSendTransaction([instruction], [delegatee]);
    return { signature: sig };
  }

  async transferFixed(
    delegatee: TransactionSigner,
    delegator: Address,
    delegatorAta: Address,
    tokenMint: Address,
    delegationPda: Address,
    amount: number | bigint,
    receiverAta: Address,
  ): Promise<{ signature: string }> {
    return this.transfer(
      'fixed',
      delegatee,
      delegator,
      delegatorAta,
      tokenMint,
      delegationPda,
      amount,
      receiverAta,
    );
  }

  async transferRecurring(
    delegatee: TransactionSigner,
    delegator: Address,
    delegatorAta: Address,
    tokenMint: Address,
    delegationPda: Address,
    amount: number | bigint,
    receiverAta: Address,
  ): Promise<{ signature: string }> {
    return this.transfer(
      'recurring',
      delegatee,
      delegator,
      delegatorAta,
      tokenMint,
      delegationPda,
      amount,
      receiverAta,
    );
  }

  async getDelegationsForWallet(wallet: Address): Promise<Delegation[]> {
    const response = await this.client.rpc
      .getProgramAccounts(MULTI_DELEGATOR_PROGRAM_ADDRESS, {
        encoding: 'base64',
        filters: [
          {
            memcmp: {
              offset: BigInt(DELEGATOR_OFFSET),
              bytes:
                wallet as unknown as import('@solana/rpc-types').Base58EncodedBytes,
              encoding: 'base58',
            },
          },
        ],
      })
      .send();

    const delegations: Delegation[] = [];
    const base64Encoder = getBase64Encoder();

    for (const account of response) {
      const base64Data = account.account.data[0];
      const data = base64Encoder.encode(base64Data);
      const kind = data[KIND_DISCRIMINATOR_OFFSET];

      const encodedAccount = {
        address: account.pubkey,
        data,
        executable: account.account.executable,
        lamports: account.account.lamports,
        programAddress: account.account.owner,
        space: account.account.space,
      };

      if (kind === DelegationKind.Fixed) {
        const decoded = decodeFixedDelegation(encodedAccount);
        delegations.push({
          kind: 'fixed',
          address: account.pubkey,
          data: decoded.data,
        });
      } else if (kind === DelegationKind.Recurring) {
        const decoded = decodeRecurringDelegation(encodedAccount);
        delegations.push({
          kind: 'recurring',
          address: account.pubkey,
          data: decoded.data,
        });
      }
    }

    return delegations;
  }

  /**
   * Check if the MultiDelegate PDA is initialized for a user and token mint.
   * The MultiDelegate account must be initialized before creating delegations.
   * Initialization also sets up SPL token delegation to the PDA.
   *
   * @param user - User's wallet address
   * @param tokenMint - Token mint address
   * @returns Object with initialized status and PDA address
   */
  async isMultiDelegateInitialized(
    user: Address,
    tokenMint: Address,
  ): Promise<{ initialized: boolean; pda: Address }> {
    const [pda] = await getMultiDelegatePDA(user, tokenMint);
    const account = await fetchMaybeMultiDelegate(this.client.rpc, pda);
    return { initialized: account !== null, pda };
  }
}
