import { findAssociatedTokenPda } from '@solana-program/token';
import type {
  Address,
  Base58EncodedBytes,
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
import {
  DELEGATOR_OFFSET,
  DISCRIMINATOR_OFFSET,
  MAX_PLAN_DESTINATIONS,
  MAX_PLAN_PULLERS,
  METADATA_URI_LEN,
  PLAN_OWNER_OFFSET,
  PLAN_SIZE,
  ZERO_ADDRESS,
} from './constants.js';
import {
  AccountDiscriminator,
  decodeFixedDelegation,
  decodePlan,
  decodeRecurringDelegation,
  decodeSubscriptionDelegation,
  type FixedDelegation,
  fetchMaybeMultiDelegate,
  getCancelSubscriptionInstructionAsync,
  getCloseMultiDelegateInstruction,
  getCreateFixedDelegationInstruction,
  getCreatePlanInstruction,
  getCreateRecurringDelegationInstruction,
  getDeletePlanInstruction,
  getInitMultiDelegateInstruction,
  getRevokeDelegationInstruction,
  getSubscribeInstructionAsync,
  getTransferFixedInstructionAsync,
  getTransferRecurringInstructionAsync,
  getTransferSubscriptionInstructionAsync,
  getUpdatePlanInstruction,
  MULTI_DELEGATOR_PROGRAM_ADDRESS,
  type Plan,
  type PlanStatus,
  type RecurringDelegation,
  type SubscriptionDelegation,
} from './generated/index.js';
import {
  getDelegationPDA,
  getMultiDelegatePDA,
  getPlanPDA,
  getSubscriptionPDA,
} from './pdas.js';

type SolanaClient = {
  rpc: Rpc<GetAccountInfoApi & GetLatestBlockhashApi & GetProgramAccountsApi>;
  sendAndConfirmTransaction: (
    tx: Awaited<ReturnType<typeof signTransactionMessageWithSigners>>,
  ) => Promise<string>;
};

export type Delegation =
  | { kind: 'fixed'; address: Address; data: FixedDelegation }
  | { kind: 'recurring'; address: Address; data: RecurringDelegation }
  | {
      kind: 'subscription';
      address: Address;
      data: SubscriptionDelegation;
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
    tokenProgram: Address,
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

  async closeMultiDelegate(
    user: TransactionSigner,
    tokenMint: Address,
  ): Promise<{ signature: string }> {
    const [multiDelegate] = await getMultiDelegatePDA(user.address, tokenMint);

    const instruction = getCloseMultiDelegateInstruction({
      user,
      multiDelegate,
    });

    const sig = await this.buildAndSendTransaction([instruction], [user]);
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
      fixedDelegation: {
        nonce,
        amount,
        expiryTs,
      },
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
      recurringDelegation: {
        nonce,
        amountPerPeriod,
        periodLengthS,
        startTs,
        expiryTs,
      },
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
    tokenProgram: Address,
  ): Promise<{ signature: string }> {
    const [multiDelegate] = await getMultiDelegatePDA(delegator, tokenMint);

    const transferParams = {
      delegationPda,
      multiDelegate,
      delegatorAta,
      receiverAta,
      tokenProgram,
      delegatee,
      transferData: {
        amount,
        delegator,
        mint: tokenMint,
      },
    };

    const instruction =
      kind === 'fixed'
        ? await getTransferFixedInstructionAsync(transferParams)
        : await getTransferRecurringInstructionAsync(transferParams);

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
    tokenProgram: Address,
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
      tokenProgram,
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
    tokenProgram: Address,
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
      tokenProgram,
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
              bytes: wallet as string as Base58EncodedBytes,
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
      const kind = data[DISCRIMINATOR_OFFSET];

      const encodedAccount = {
        address: account.pubkey,
        data,
        executable: account.account.executable,
        lamports: account.account.lamports,
        programAddress: account.account.owner,
        space: account.account.space,
      };

      switch (kind) {
        case AccountDiscriminator.FixedDelegation: {
          const decoded = decodeFixedDelegation(encodedAccount);
          delegations.push({
            kind: 'fixed',
            address: account.pubkey,
            data: decoded.data,
          });
          break;
        }
        case AccountDiscriminator.RecurringDelegation: {
          const decoded = decodeRecurringDelegation(encodedAccount);
          delegations.push({
            kind: 'recurring',
            address: account.pubkey,
            data: decoded.data,
          });
          break;
        }
        case AccountDiscriminator.SubscriptionDelegation: {
          const decoded = decodeSubscriptionDelegation(encodedAccount);
          delegations.push({
            kind: 'subscription',
            address: account.pubkey,
            data: decoded.data,
          });
          break;
        }
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

  async createPlan(
    owner: TransactionSigner,
    planId: number | bigint,
    mint: Address,
    amount: number | bigint,
    periodHours: number | bigint,
    endTs: number | bigint,
    destinations: Address[],
    pullers: Address[],
    metadataUri: string,
  ): Promise<{ signature: string; planPda: Address }> {
    if (destinations.length > MAX_PLAN_DESTINATIONS)
      throw new Error(
        `destinations must have at most ${MAX_PLAN_DESTINATIONS} entries`,
      );
    if (pullers.length > MAX_PLAN_PULLERS)
      throw new Error(`pullers must have at most ${MAX_PLAN_PULLERS} entries`);

    const uriBytes = new TextEncoder().encode(metadataUri);
    if (uriBytes.length > METADATA_URI_LEN)
      throw new Error(`metadataUri exceeds ${METADATA_URI_LEN} bytes`);

    const paddedDestinations: Address[] = Array.from(
      { length: MAX_PLAN_DESTINATIONS },
      (_, i) => destinations[i] || ZERO_ADDRESS,
    );

    const paddedPullers: Address[] = Array.from(
      { length: MAX_PLAN_PULLERS },
      (_, i) => pullers[i] || ZERO_ADDRESS,
    );

    const [planPda] = await getPlanPDA(owner.address, planId);

    const instruction = getCreatePlanInstruction({
      merchant: owner,
      planPda,
      tokenMint: mint,
      planData: {
        planId,
        mint,
        amount,
        periodHours,
        endTs,
        destinations: paddedDestinations,
        pullers: paddedPullers,
        metadataUri,
      },
    });

    const sig = await this.buildAndSendTransaction([instruction], [owner]);
    return { signature: sig, planPda };
  }

  async updatePlan(
    owner: TransactionSigner,
    planPda: Address,
    status: PlanStatus,
    endTs: number | bigint,
    metadataUri: string,
  ): Promise<{ signature: string }> {
    const uriBytes = new TextEncoder().encode(metadataUri);
    if (uriBytes.length > METADATA_URI_LEN)
      throw new Error(`metadataUri exceeds ${METADATA_URI_LEN} bytes`);

    if (endTs !== 0 && endTs !== BigInt(0)) {
      const endTsNum = typeof endTs === 'bigint' ? Number(endTs) : endTs;
      if (endTsNum <= Math.floor(Date.now() / 1000))
        throw new Error('endTs must be in the future');
    }

    const instruction = getUpdatePlanInstruction({
      owner,
      planPda,
      updatePlanData: { status, endTs, metadataUri },
    });

    const signature = await this.buildAndSendTransaction(
      [instruction],
      [owner],
    );
    return { signature };
  }

  async subscribe(
    subscriber: TransactionSigner,
    merchant: Address,
    planId: number | bigint,
    tokenMint: Address,
  ): Promise<{ signature: string; subscriptionPda: Address }> {
    const [planPda, planBump] = await getPlanPDA(merchant, planId);
    const [multiDelegatePda] = await getMultiDelegatePDA(
      subscriber.address,
      tokenMint,
    );
    const [subscriptionPda] = await getSubscriptionPDA(
      planPda,
      subscriber.address,
    );
    const instruction = await getSubscribeInstructionAsync({
      subscriber,
      merchant,
      planPda,
      subscriptionPda,
      multiDelegatePda,
      subscribeData: {
        planId,
        planBump,
      },
    });

    const signature = await this.buildAndSendTransaction(
      [instruction],
      [subscriber],
    );
    return { signature, subscriptionPda };
  }

  async cancelSubscription(
    subscriber: TransactionSigner,
    planPda: Address,
    subscriptionPda: Address,
  ): Promise<{ signature: string }> {
    const instruction = await getCancelSubscriptionInstructionAsync({
      subscriber,
      planPda,
      subscriptionPda,
    });

    const signature = await this.buildAndSendTransaction(
      [instruction],
      [subscriber],
    );
    return { signature };
  }

  async transferSubscription(
    caller: TransactionSigner,
    delegator: Address,
    tokenMint: Address,
    subscriptionPda: Address,
    planPda: Address,
    amount: number | bigint,
    receiverAta: Address,
    tokenProgram: Address,
  ): Promise<{ signature: string }> {
    const [multiDelegate] = await getMultiDelegatePDA(delegator, tokenMint);
    const [delegatorAta] = await findAssociatedTokenPda({
      mint: tokenMint,
      owner: delegator,
      tokenProgram,
    });

    const instruction = await getTransferSubscriptionInstructionAsync({
      subscriptionPda,
      planPda,
      multiDelegate,
      delegatorAta,
      receiverAta,
      caller,
      tokenProgram,
      transferData: {
        amount,
        delegator,
        mint: tokenMint,
      },
    });

    const signature = await this.buildAndSendTransaction(
      [instruction],
      [caller],
    );
    return { signature };
  }

  async deletePlan(
    owner: TransactionSigner,
    planPda: Address,
  ): Promise<{ signature: string }> {
    const instruction = getDeletePlanInstruction({ owner, planPda });
    const signature = await this.buildAndSendTransaction(
      [instruction],
      [owner],
    );
    return { signature };
  }

  async getPlansForOwner(
    owner: Address,
  ): Promise<Array<{ address: Address; data: Plan }>> {
    const response = await this.client.rpc
      .getProgramAccounts(MULTI_DELEGATOR_PROGRAM_ADDRESS, {
        encoding: 'base64',
        filters: [
          { dataSize: BigInt(PLAN_SIZE) },
          {
            memcmp: {
              offset: BigInt(PLAN_OWNER_OFFSET),
              bytes: owner as string as Base58EncodedBytes,
              encoding: 'base58',
            },
          },
        ],
      })
      .send();

    const base64Encoder = getBase64Encoder();
    return response.map((account) => {
      const data = base64Encoder.encode(account.account.data[0]);
      const decoded = decodePlan({
        address: account.pubkey,
        data,
        executable: account.account.executable,
        lamports: account.account.lamports,
        programAddress: account.account.owner,
        space: account.account.space,
      });
      return { address: account.pubkey, data: decoded.data };
    });
  }
}
