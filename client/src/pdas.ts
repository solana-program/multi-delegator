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

export async function getFixedDelegatePDA(
  multiDelegate: Address,
  delegate: Address,
  payer: Address,
  kind: number,
): Promise<[Address, number]> {
  const seeds = [
    new TextEncoder().encode('Delegate'),
    addressEncoder.encode(multiDelegate),
    addressEncoder.encode(delegate),
    addressEncoder.encode(payer),
    new Uint8Array([kind]),
  ];

  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: MULTI_DELEGATOR_PROGRAM_ADDRESS,
    seeds,
  });
  return [pda, bump];
}
