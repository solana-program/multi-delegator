import type { Address, Instruction, TransactionSigner } from 'gill';
import {
  getCancelSubscriptionInstructionAsync,
  getSubscribeInstructionAsync,
} from '../generated/index.js';
import {
  getMultiDelegatePDA,
  getPlanPDA,
  getSubscriptionPDA,
} from '../pdas.js';

/**
 * Builds a `subscribe` instruction, deriving Plan, MultiDelegate, and Subscription PDAs.
 *
 * @param params.subscriber - The wallet subscribing to the plan.
 * @param params.merchant - The plan owner's address.
 * @param params.planId - Numeric identifier of the plan to subscribe to.
 * @param params.tokenMint - SPL token mint the plan uses.
 * @returns The instruction array and the derived `subscriptionPda`.
 */
export async function buildSubscribe(params: {
  subscriber: TransactionSigner;
  merchant: Address;
  planId: number | bigint;
  tokenMint: Address;
  programAddress?: Address;
}): Promise<{ instructions: Instruction[]; subscriptionPda: Address }> {
  const { subscriber, merchant, planId, tokenMint, programAddress } = params;
  const config = programAddress ? { programAddress } : undefined;

  const [planPda, planBump] = await getPlanPDA(
    merchant,
    planId,
    programAddress,
  );
  const [multiDelegatePda] = await getMultiDelegatePDA(
    subscriber.address,
    tokenMint,
    programAddress,
  );
  const [subscriptionPda] = await getSubscriptionPDA(
    planPda,
    subscriber.address,
    programAddress,
  );

  const instruction = await getSubscribeInstructionAsync(
    {
      subscriber,
      merchant,
      planPda,
      subscriptionPda,
      multiDelegatePda,
      subscribeData: { planId, planBump },
    },
    config,
  );

  return { instructions: [instruction], subscriptionPda };
}

/**
 * Builds a `cancelSubscription` instruction that marks a subscription for expiry.
 *
 * @param params.subscriber - The wallet that owns the subscription.
 * @param params.planPda - Address of the associated plan account.
 * @param params.subscriptionPda - Address of the subscription account to cancel.
 * @returns The instruction array.
 */
export async function buildCancelSubscription(params: {
  subscriber: TransactionSigner;
  planPda: Address;
  subscriptionPda: Address;
  programAddress?: Address;
}): Promise<{ instructions: Instruction[] }> {
  const config = params.programAddress
    ? { programAddress: params.programAddress }
    : undefined;
  const instruction = await getCancelSubscriptionInstructionAsync(
    {
      subscriber: params.subscriber,
      planPda: params.planPda,
      subscriptionPda: params.subscriptionPda,
    },
    config,
  );

  return { instructions: [instruction] };
}
