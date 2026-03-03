import {
  type Address,
  getAddressEncoder,
  getProgramDerivedAddress,
} from 'gill';
import {
  DELEGATION_SEED,
  EVENT_AUTHORITY_SEED,
  MULTI_DELEGATE_SEED,
  PLAN_SEED,
  PROGRAM_ID,
  SUBSCRIPTION_SEED,
  U64_BYTE_SIZE,
} from './constants.js';

const addressEncoder = getAddressEncoder();
const textEncoder = new TextEncoder();

function encodeU64Le(value: number | bigint): Uint8Array {
  const bytes = new Uint8Array(U64_BYTE_SIZE);
  new DataView(bytes.buffer).setBigUint64(0, BigInt(value), true);
  return bytes;
}

export async function getMultiDelegatePDA(
  user: Address,
  tokenMint: Address,
  programId?: Address,
): Promise<[Address, number]> {
  const seeds = [
    textEncoder.encode(MULTI_DELEGATE_SEED),
    addressEncoder.encode(user),
    addressEncoder.encode(tokenMint),
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: programId ?? PROGRAM_ID,
    seeds,
  });
  return [pda, bump];
}

export async function getDelegationPDA(
  multiDelegate: Address,
  delegator: Address,
  delegatee: Address,
  nonce: number | bigint,
  programId?: Address,
): Promise<[Address, number]> {
  const seeds = [
    textEncoder.encode(DELEGATION_SEED),
    addressEncoder.encode(multiDelegate),
    addressEncoder.encode(delegator),
    addressEncoder.encode(delegatee),
    encodeU64Le(nonce),
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: programId ?? PROGRAM_ID,
    seeds,
  });
  return [pda, bump];
}

export async function getPlanPDA(
  owner: Address,
  planId: number | bigint,
  programId?: Address,
): Promise<[Address, number]> {
  const seeds = [
    textEncoder.encode(PLAN_SEED),
    addressEncoder.encode(owner),
    encodeU64Le(planId),
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: programId ?? PROGRAM_ID,
    seeds,
  });
  return [pda, bump];
}

export async function getEventAuthorityPDA(
  programId?: Address,
): Promise<[Address, number]> {
  const seeds = [textEncoder.encode(EVENT_AUTHORITY_SEED)];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: programId ?? PROGRAM_ID,
    seeds,
  });
  return [pda, bump];
}

export async function getSubscriptionPDA(
  planPda: Address,
  subscriber: Address,
  programId?: Address,
): Promise<[Address, number]> {
  const seeds = [
    textEncoder.encode(SUBSCRIPTION_SEED),
    addressEncoder.encode(planPda),
    addressEncoder.encode(subscriber),
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: programId ?? PROGRAM_ID,
    seeds,
  });
  return [pda, bump];
}
