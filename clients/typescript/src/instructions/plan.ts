import type { Address, Instruction, TransactionSigner } from 'gill';
import {
  MAX_PLAN_DESTINATIONS,
  MAX_PLAN_PULLERS,
  METADATA_URI_LEN,
  ZERO_ADDRESS,
} from '../constants.js';
import { ValidationError } from '../errors/types.js';
import type { PlanStatus } from '../generated/index.js';
import {
  getCreatePlanInstruction,
  getDeletePlanInstruction,
  getUpdatePlanInstruction,
} from '../generated/index.js';
import { getPlanPDA } from '../pdas.js';

const textEncoder = new TextEncoder();

function validateMetadataUri(metadataUri: string): void {
  if (textEncoder.encode(metadataUri).length > METADATA_URI_LEN)
    throw new ValidationError(`metadataUri exceeds ${METADATA_URI_LEN} bytes`);
}

function padAddresses(addresses: Address[], maxLen: number): Address[] {
  return Array.from({ length: maxLen }, (_, i) => addresses[i] ?? ZERO_ADDRESS);
}

/**
 * Builds a `createPlan` instruction with input validation, array padding, and auto-PDA derivation.
 *
 * @param params.owner - The merchant wallet that owns the plan.
 * @param params.planId - Unique numeric identifier for this plan under the owner.
 * @param params.mint - SPL token mint the plan accepts.
 * @param params.amount - Token amount charged per billing period.
 * @param params.periodHours - Billing period length in hours.
 * @param params.endTs - Unix timestamp when the plan stops accepting new subscriptions (0 for no end).
 * @param params.destinations - Recipient addresses for transferred funds (max {@link MAX_PLAN_DESTINATIONS}).
 * @param params.pullers - Addresses authorized to execute subscription pulls (max {@link MAX_PLAN_PULLERS}).
 * @param params.metadataUri - Off-chain metadata URI (max {@link METADATA_URI_LEN} bytes).
 * @returns The instruction array and the derived `planPda`.
 * @throws {ValidationError} If inputs fail validation checks.
 */
export async function buildCreatePlan(params: {
  owner: TransactionSigner;
  planId: number | bigint;
  mint: Address;
  amount: number | bigint;
  periodHours: number | bigint;
  endTs: number | bigint;
  destinations: Address[];
  pullers: Address[];
  metadataUri: string;
  tokenProgram?: Address;
  programAddress?: Address;
}): Promise<{ instructions: Instruction[]; planPda: Address }> {
  const {
    owner,
    planId,
    mint,
    amount,
    periodHours,
    endTs,
    destinations,
    pullers,
    metadataUri,
    tokenProgram,
    programAddress,
  } = params;
  const config = programAddress ? { programAddress } : undefined;

  if (BigInt(amount) <= 0n)
    throw new ValidationError('amount must be greater than zero');
  if (BigInt(periodHours) <= 0n)
    throw new ValidationError('periodHours must be greater than zero');
  if (destinations.length > MAX_PLAN_DESTINATIONS)
    throw new ValidationError(
      `destinations must have at most ${MAX_PLAN_DESTINATIONS} entries`,
    );
  if (pullers.length > MAX_PLAN_PULLERS)
    throw new ValidationError(
      `pullers must have at most ${MAX_PLAN_PULLERS} entries`,
    );
  validateMetadataUri(metadataUri);

  const paddedDestinations = padAddresses(destinations, MAX_PLAN_DESTINATIONS);
  const paddedPullers = padAddresses(pullers, MAX_PLAN_PULLERS);

  const [planPda] = await getPlanPDA(owner.address, planId, programAddress);

  const instruction = getCreatePlanInstruction(
    {
      merchant: owner,
      planPda,
      tokenMint: mint,
      tokenProgram,
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
    },
    config,
  );

  return { instructions: [instruction], planPda };
}

/**
 * Builds an `updatePlan` instruction with input validation and puller array padding.
 *
 * @param params.owner - The merchant wallet that owns the plan.
 * @param params.planPda - Address of the plan account to update.
 * @param params.status - New plan status (e.g. Active, Paused).
 * @param params.endTs - New end timestamp (0 to remove end date).
 * @param params.metadataUri - Updated metadata URI (max {@link METADATA_URI_LEN} bytes).
 * @param params.pullers - Updated puller addresses (max {@link MAX_PLAN_PULLERS}), defaults to empty.
 * @returns The instruction array.
 * @throws {ValidationError} If metadataUri exceeds byte limit or pullers exceed max count.
 */
export function buildUpdatePlan(params: {
  owner: TransactionSigner;
  planPda: Address;
  status: PlanStatus;
  endTs: number | bigint;
  metadataUri: string;
  pullers?: Address[];
  programAddress?: Address;
}): { instructions: Instruction[] } {
  const { owner, planPda, status, endTs, metadataUri, programAddress } = params;
  const config = programAddress ? { programAddress } : undefined;
  const pullers = params.pullers ?? [];

  validateMetadataUri(metadataUri);

  if (pullers.length > MAX_PLAN_PULLERS)
    throw new ValidationError(`pullers exceeds max of ${MAX_PLAN_PULLERS}`);

  const paddedPullers = padAddresses(pullers, MAX_PLAN_PULLERS) as [
    Address,
    Address,
    Address,
    Address,
  ];

  const instruction = getUpdatePlanInstruction(
    {
      owner,
      planPda,
      updatePlanData: {
        status,
        endTs,
        pullers: paddedPullers,
        metadataUri,
      },
    },
    config,
  );

  return { instructions: [instruction] };
}

/**
 * Builds a `deletePlan` instruction that closes the plan account and reclaims its rent.
 *
 * @param params.owner - The merchant wallet that owns the plan.
 * @param params.planPda - Address of the plan account to delete.
 * @returns The instruction array.
 */
export function buildDeletePlan(params: {
  owner: TransactionSigner;
  planPda: Address;
  programAddress?: Address;
}): { instructions: Instruction[] } {
  const config = params.programAddress
    ? { programAddress: params.programAddress }
    : undefined;
  const instruction = getDeletePlanInstruction(
    {
      owner: params.owner,
      planPda: params.planPda,
    },
    config,
  );
  return { instructions: [instruction] };
}
