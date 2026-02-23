import { useQuery } from '@tanstack/react-query'
import { useWalletUi } from '@wallet-ui/react'
import { createSolanaRpc, address } from 'gill'
import type { Address } from 'gill'
import {
  MULTI_DELEGATOR_PROGRAM_ADDRESS,
  SUBSCRIPTION_SIZE,
  DELEGATOR_OFFSET,
  decodeSubscriptionDelegation,
  fetchAllMaybePlan,
} from '@multidelegator/client'
import type { SubscriptionDelegation, Plan } from '@multidelegator/client'
import { useClusterConfig } from '@/hooks/use-cluster-config'
import type { DelegationAccountRaw } from '@/lib/types'
import { decodeBase64ToUint8Array } from '@/lib/utils'

export interface EnrichedSubscription {
  address: string
  subscription: SubscriptionDelegation
  plan: Plan | null
}

async function fetchMySubscriptions(rpcUrl: string, walletAddress: string): Promise<EnrichedSubscription[]> {
  const rpc = createSolanaRpc(rpcUrl)

  const response = await rpc
    .getProgramAccounts(address(MULTI_DELEGATOR_PROGRAM_ADDRESS), {
      filters: [
        { dataSize: BigInt(SUBSCRIPTION_SIZE) },
        {
          memcmp: {
            offset: BigInt(DELEGATOR_OFFSET),
            bytes: walletAddress,
            encoding: 'base58',
          },
        },
      ],
      encoding: 'base64',
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any)
    .send()

  const accounts = response as unknown as DelegationAccountRaw[]
  if (accounts.length === 0) return []

  const subs: { address: string; subscription: SubscriptionDelegation }[] = []

  for (const entry of accounts) {
    try {
      const [base64Data] = entry.account.data
      const data = decodeBase64ToUint8Array(base64Data)

      const encodedAccount = {
        address: entry.pubkey,
        data,
        executable: entry.account.executable,
        lamports: entry.account.lamports,
        owner: entry.account.owner,
        programAddress: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
        space: BigInt(data.length),
      }

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const decoded = decodeSubscriptionDelegation(encodedAccount as any)
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      subs.push({ address: entry.pubkey, subscription: (decoded as any).data })
    } catch {
      console.warn('Failed to decode subscription account:', entry.pubkey)
    }
  }

  const planAddresses = [...new Set(subs.map((s) => s.subscription.header.delegatee))]
  const maybePlans = await fetchAllMaybePlan(rpc, planAddresses as Address[])

  const planMap = new Map<string, Plan>()
  for (const mp of maybePlans) {
    if (mp.exists) planMap.set(mp.address, mp.data)
  }

  return subs.map((s) => ({
    address: s.address,
    subscription: s.subscription,
    plan: planMap.get(s.subscription.header.delegatee) ?? null,
  }))
}

export function useMySubscriptions() {
  const { account } = useWalletUi()
  const clusterConfig = useClusterConfig()

  return useQuery({
    queryKey: ['subscriptions', 'my', account?.address, clusterConfig.id],
    queryFn: () => fetchMySubscriptions(clusterConfig.url, account!.address),
    enabled: !!account?.address,
  })
}
