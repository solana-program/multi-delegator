import { findAssociatedTokenPda } from '@solana-program/token';
import type { Address } from '@solana/kit';

export {
  getDelegationPDA,
  getEventAuthorityPDA,
  getMultiDelegatePDA,
  getPlanPDA,
  getSubscriptionPDA,
} from '@multidelegator/client';

export async function getAssociatedTokenAddress(
  owner: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<Address> {
  const [ata] = await findAssociatedTokenPda({ mint, owner, tokenProgram });
  return ata;
}
