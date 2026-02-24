import { useQuery } from '@tanstack/react-query'
import { useWalletUi } from '@wallet-ui/react'
import { createSolanaRpc, address } from 'gill'
import type { Address } from 'gill'
import {
  MULTI_DELEGATOR_PROGRAM_ADDRESS,
  SUBSCRIPTION_SIZE,
  DELEGATOR_OFFSET,
  DELEGATEE_OFFSET,
  decodeSubscriptionDelegation,
  fetchAllMaybePlan,
} from '@multidelegator/client'
import type { SubscriptionDelegation, Plan } from '@multidelegator/client'
import { useClusterConfig } from '@/hooks/use-cluster-config'
import type { DelegationAccountRaw } from '@/lib/types'
import { decodeBase64ToUint8Array } from '@/lib/utils'

export interface PlanSubscriber {
  subscriptionAddress: string
  delegator: string
  amountPulledInPeriod: bigint
  currentPeriodStartTs: bigint
  expiresAtTs: bigint
}

function decodeSubscriptionFromRaw(entry: DelegationAccountRaw): SubscriptionDelegation {
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
  return (decoded as any).data as SubscriptionDelegation
}

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
      subs.push({ address: entry.pubkey, subscription: decodeSubscriptionFromRaw(entry) })
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

export async function fetchPlanSubscriptions(rpcUrl: string, planAddress: string): Promise<PlanSubscriber[]> {
  const rpc = createSolanaRpc(rpcUrl)

  const response = await rpc
    .getProgramAccounts(address(MULTI_DELEGATOR_PROGRAM_ADDRESS), {
      filters: [
        { dataSize: BigInt(SUBSCRIPTION_SIZE) },
        {
          memcmp: {
            offset: BigInt(DELEGATEE_OFFSET),
            bytes: planAddress,
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

  const subscribers: PlanSubscriber[] = []

  for (const entry of accounts) {
    try {
      const sub = decodeSubscriptionFromRaw(entry)
      subscribers.push({
        subscriptionAddress: entry.pubkey,
        delegator: sub.header.delegator,
        amountPulledInPeriod: sub.amountPulledInPeriod,
        currentPeriodStartTs: sub.currentPeriodStartTs,
        expiresAtTs: sub.expiresAtTs,
      })
    } catch {
      console.warn('Failed to decode subscription account:', entry.pubkey)
    }
  }

  return subscribers
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

async function fetchSubscriberCount(rpcUrl: string, planAddress: string): Promise<number> {
  const rpc = createSolanaRpc(rpcUrl)

  const response = await rpc
    .getProgramAccounts(address(MULTI_DELEGATOR_PROGRAM_ADDRESS), {
      filters: [
        { dataSize: BigInt(SUBSCRIPTION_SIZE) },
        {
          memcmp: {
            offset: BigInt(DELEGATEE_OFFSET),
            bytes: planAddress,
            encoding: 'base58',
          },
        },
      ],
      dataSlice: { offset: 0, length: 0 },
      encoding: 'base64',
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any)
    .send()

  return (response as unknown as DelegationAccountRaw[]).length
}

export function useSubscriberCount(planAddress: string | null) {
  const clusterConfig = useClusterConfig()

  return useQuery({
    queryKey: ['subscriberCount', planAddress, clusterConfig.id],
    queryFn: () => fetchSubscriberCount(clusterConfig.url, planAddress!),
    enabled: !!planAddress,
  })
}

async function fetchSubscriberCounts(rpcUrl: string, planAddresses: string[]): Promise<Map<string, number>> {
  const counts = await Promise.all(
    planAddresses.map((addr) => fetchSubscriberCount(rpcUrl, addr))
  )
  const map = new Map<string, number>()
  planAddresses.forEach((addr, i) => map.set(addr, counts[i]))
  return map
}

export function useSubscriberCounts(planAddresses: string[]) {
  const clusterConfig = useClusterConfig()
  const key = planAddresses.slice().sort().join(',')

  return useQuery({
    queryKey: ['subscriberCounts', key, clusterConfig.id],
    queryFn: () => fetchSubscriberCounts(clusterConfig.url, planAddresses),
    enabled: planAddresses.length > 0,
  })
}
