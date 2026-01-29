import {
  type Address,
  getAddressEncoder,
  getProgramDerivedAddress,
} from 'gill';
import { MULTI_DELEGATOR_PROGRAM_ADDRESS } from './generated/index.js';

const addressEncoder = getAddressEncoder();

export async function getMultiDelegatePDA(
  user: Address,
  tokenMint: Address,
): Promise<[Address, number]> {
  const seeds = [
    new TextEncoder().encode('MultiDelegate'),
    addressEncoder.encode(user),
    addressEncoder.encode(tokenMint),
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: MULTI_DELEGATOR_PROGRAM_ADDRESS,
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
  const nonceBytes = new Uint8Array(8);
  new DataView(nonceBytes.buffer).setBigUint64(0, BigInt(nonce), true);

  const seeds = [
    new TextEncoder().encode('delegation'),
    addressEncoder.encode(multiDelegate),
    addressEncoder.encode(delegator),
    addressEncoder.encode(delegatee),
    nonceBytes,
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: MULTI_DELEGATOR_PROGRAM_ADDRESS,
    seeds,
  });
  return [pda, bump];
}
