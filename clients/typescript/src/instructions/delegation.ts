import type { Address, Instruction, TransactionSigner } from 'gill';
import { ValidationError } from '../errors/types.js';
import {
  getCloseMultiDelegateInstruction,
  getCreateFixedDelegationInstruction,
  getCreateRecurringDelegationInstruction,
  getInitMultiDelegateInstruction,
  getRevokeDelegationInstruction,
} from '../generated/index.js';
import { getDelegationPDA, getMultiDelegatePDA } from '../pdas.js';

/**
 * Builds an `initMultiDelegate` instruction, deriving the MultiDelegate PDA automatically.
 *
 * @param params.owner - The wallet that owns the multi-delegate account.
 * @param params.tokenMint - SPL token mint address.
 * @param params.userAta - Owner's associated token account for the mint.
 * @param params.tokenProgram - Token program (typically Token-2022).
 * @returns The instruction array and the derived `multiDelegatePda`.
 */
export async function buildInitMultiDelegate(params: {
  owner: TransactionSigner;
  tokenMint: Address;
  userAta: Address;
  tokenProgram: Address;
  programAddress?: Address;
}): Promise<{ instructions: Instruction[]; multiDelegatePda: Address }> {
  const { owner, tokenMint, userAta, tokenProgram, programAddress } = params;
  const config = programAddress ? { programAddress } : undefined;
  const [multiDelegatePda] = await getMultiDelegatePDA(
    owner.address,
    tokenMint,
    programAddress,
  );

  const instruction = getInitMultiDelegateInstruction(
    {
      owner,
      multiDelegate: multiDelegatePda,
      tokenMint,
      userAta,
      tokenProgram,
    },
    config,
  );

  return { instructions: [instruction], multiDelegatePda };
}

/**
 * Builds a `createFixedDelegation` instruction, deriving MultiDelegate and Delegation PDAs.
 *
 * @param params.delegator - The wallet creating the delegation.
 * @param params.tokenMint - SPL token mint address.
 * @param params.delegatee - Address authorized to pull tokens.
 * @param params.nonce - Unique nonce distinguishing multiple delegations to the same delegatee.
 * @param params.amount - Total token amount the delegatee may transfer.
 * @param params.expiryTs - Unix timestamp after which the delegation expires (0 for no expiry).
 * @returns The instruction array and the derived `delegationPda`.
 * @throws {ValidationError} If amount is zero or negative.
 */
export async function buildCreateFixedDelegation(params: {
  delegator: TransactionSigner;
  tokenMint: Address;
  delegatee: Address;
  nonce: number | bigint;
  amount: number | bigint;
  expiryTs: number | bigint;
  programAddress?: Address;
}): Promise<{ instructions: Instruction[]; delegationPda: Address }> {
  const {
    delegator,
    tokenMint,
    delegatee,
    nonce,
    amount,
    expiryTs,
    programAddress,
  } = params;
  const config = programAddress ? { programAddress } : undefined;

  if (BigInt(amount) <= 0n)
    throw new ValidationError('amount must be greater than zero');

  const [multiDelegate] = await getMultiDelegatePDA(
    delegator.address,
    tokenMint,
    programAddress,
  );
  const [delegationPda] = await getDelegationPDA(
    multiDelegate,
    delegator.address,
    delegatee,
    nonce,
    programAddress,
  );

  const instruction = getCreateFixedDelegationInstruction(
    {
      delegator,
      multiDelegate,
      delegationAccount: delegationPda,
      delegatee,
      fixedDelegation: { nonce, amount, expiryTs },
    },
    config,
  );

  return { instructions: [instruction], delegationPda };
}

/**
 * Builds a `createRecurringDelegation` instruction, deriving MultiDelegate and Delegation PDAs.
 *
 * @param params.delegator - The wallet creating the delegation.
 * @param params.tokenMint - SPL token mint address.
 * @param params.delegatee - Address authorized to pull tokens each period.
 * @param params.nonce - Unique nonce distinguishing multiple delegations to the same delegatee.
 * @param params.amountPerPeriod - Token amount the delegatee may transfer per period.
 * @param params.periodLengthS - Period length in seconds.
 * @param params.startTs - Unix timestamp when the first period begins.
 * @param params.expiryTs - Unix timestamp after which the delegation expires (0 for no expiry).
 * @returns The instruction array and the derived `delegationPda`.
 * @throws {ValidationError} If amountPerPeriod or periodLengthS is zero or negative.
 */
export async function buildCreateRecurringDelegation(params: {
  delegator: TransactionSigner;
  tokenMint: Address;
  delegatee: Address;
  nonce: number | bigint;
  amountPerPeriod: number | bigint;
  periodLengthS: number | bigint;
  startTs: number | bigint;
  expiryTs: number | bigint;
  programAddress?: Address;
}): Promise<{ instructions: Instruction[]; delegationPda: Address }> {
  const {
    delegator,
    tokenMint,
    delegatee,
    nonce,
    amountPerPeriod,
    periodLengthS,
    startTs,
    expiryTs,
    programAddress,
  } = params;
  const config = programAddress ? { programAddress } : undefined;

  if (BigInt(amountPerPeriod) <= 0n)
    throw new ValidationError('amountPerPeriod must be greater than zero');
  if (BigInt(periodLengthS) <= 0n)
    throw new ValidationError('periodLengthS must be greater than zero');

  const [multiDelegate] = await getMultiDelegatePDA(
    delegator.address,
    tokenMint,
    programAddress,
  );
  const [delegationPda] = await getDelegationPDA(
    multiDelegate,
    delegator.address,
    delegatee,
    nonce,
    programAddress,
  );

  const instruction = getCreateRecurringDelegationInstruction(
    {
      delegator,
      multiDelegate,
      delegationAccount: delegationPda,
      delegatee,
      recurringDelegation: {
        nonce,
        amountPerPeriod,
        periodLengthS,
        startTs,
        expiryTs,
      },
    },
    config,
  );

  return { instructions: [instruction], delegationPda };
}

/**
 * Builds a `revokeDelegation` instruction that permanently closes a delegation account.
 *
 * @param params.authority - The delegator or delegatee authorized to revoke.
 * @param params.delegationAccount - Address of the delegation PDA to revoke.
 * @returns The instruction array.
 */
export function buildRevokeDelegation(params: {
  authority: TransactionSigner;
  delegationAccount: Address;
  programAddress?: Address;
}): { instructions: Instruction[] } {
  const config = params.programAddress
    ? { programAddress: params.programAddress }
    : undefined;
  const instruction = getRevokeDelegationInstruction(
    {
      authority: params.authority,
      delegationAccount: params.delegationAccount,
    },
    config,
  );
  return { instructions: [instruction] };
}

/**
 * Builds a `closeMultiDelegate` instruction, deriving the MultiDelegate PDA automatically.
 * Closes the multi-delegate account and reclaims its rent.
 *
 * @param params.user - The wallet that owns the multi-delegate account.
 * @param params.tokenMint - SPL token mint associated with the account.
 * @returns The instruction array.
 */
export async function buildCloseMultiDelegate(params: {
  user: TransactionSigner;
  tokenMint: Address;
  programAddress?: Address;
}): Promise<{ instructions: Instruction[] }> {
  const config = params.programAddress
    ? { programAddress: params.programAddress }
    : undefined;
  const [multiDelegate] = await getMultiDelegatePDA(
    params.user.address,
    params.tokenMint,
    params.programAddress,
  );

  const instruction = getCloseMultiDelegateInstruction(
    {
      user: params.user,
      multiDelegate,
    },
    config,
  );

  return { instructions: [instruction] };
}
