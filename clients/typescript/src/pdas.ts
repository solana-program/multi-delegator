import {
  type Address,
  getAddressEncoder,
  getProgramDerivedAddress,
} from 'gill';
import {
  DELEGATION_SEED,
  MULTI_DELEGATE_SEED,
  PROGRAM_ID,
  U64_BYTE_SIZE,
} from './constants.js';

const addressEncoder = getAddressEncoder();
const textEncoder = new TextEncoder();

export async function getMultiDelegatePDA(
  user: Address,
  tokenMint: Address,
): Promise<[Address, number]> {
  const seeds = [
    textEncoder.encode(MULTI_DELEGATE_SEED),
    addressEncoder.encode(user),
    addressEncoder.encode(tokenMint),
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: PROGRAM_ID,
    seeds,
  });
  return [pda, bump];
}

export async function getDelegationPDA(
  multiDelegate: Address,
  delegator: Address,
  delegatee: Address,
  nonce: number | bigint,
): Promise<[Address, number]> {
  const nonceBytes = new Uint8Array(U64_BYTE_SIZE);
  new DataView(nonceBytes.buffer).setBigUint64(0, BigInt(nonce), true);

  const seeds = [
    textEncoder.encode(DELEGATION_SEED),
    addressEncoder.encode(multiDelegate),
    addressEncoder.encode(delegator),
    addressEncoder.encode(delegatee),
    nonceBytes,
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: PROGRAM_ID,
    seeds,
  });
  return [pda, bump];
}
