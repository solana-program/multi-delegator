import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useWalletUi } from '@wallet-ui/react'
import { createSolanaRpc, address } from 'gill'
import {
  MULTI_DELEGATOR_PROGRAM_ADDRESS,
  DELEGATOR_OFFSET,
  DELEGATEE_OFFSET,
  decodeFixedDelegation,
  decodeRecurringDelegation,
  getMultiDelegatePDA,
  fetchMaybeMultiDelegate,
  AccountDiscriminator,
} from '@multidelegator/client'
import type { ClusterWithUrl, DelegationAccountRaw } from '@/lib/types'
import { decodeBase64ToUint8Array } from '@/lib/utils'

export interface DelegationData {
  header: {
    delegator: string
    delegatee: string
  }
  amount: bigint
  amountPerPeriod: bigint
  periodLengthS: bigint
  expiryTs: bigint
  amountPulledInPeriod: bigint
}

export interface DelegationItem {
  address: string
  type: 'Fixed' | 'Recurring'
  data: DelegationData
}

export interface GroupedDelegations {
  fixed: DelegationItem[]
  recurring: DelegationItem[]
  all: DelegationItem[]
}

export type DelegationRole = 'delegator' | 'delegatee'

export function useMultiDelegate(tokenMint: string) {
  const { account, cluster } = useWalletUi()
  const clusterConfig = cluster as unknown as ClusterWithUrl
  const rpc = createSolanaRpc(clusterConfig.url)

  return useQuery({
    queryKey: ['multiDelegate', account?.address, tokenMint, cluster.id],
    queryFn: async () => {
      if (!account?.address) return null
      const [pda] = await getMultiDelegatePDA(address(account.address), address(tokenMint))
      return fetchMaybeMultiDelegate(rpc, pda)
    },
    enabled: !!account?.address && !!tokenMint,
  })
}

async function fetchDelegationsByRole(
  rpcUrl: string,
  walletAddress: string,
  role: DelegationRole
): Promise<GroupedDelegations> {
  const rpc = createSolanaRpc(rpcUrl)
  const offset = role === 'delegator' ? DELEGATOR_OFFSET : DELEGATEE_OFFSET

  const response = await rpc
    .getProgramAccounts(address(MULTI_DELEGATOR_PROGRAM_ADDRESS), {
      filters: [
        {
          memcmp: {
            offset: BigInt(offset),
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
  const all: DelegationItem[] = []

  for (const accountEntry of accounts) {
    const [base64Data] = accountEntry.account.data
    const data = decodeBase64ToUint8Array(base64Data)
    const kind = data[1]

    const encodedAccount = {
      address: accountEntry.pubkey,
      data: data,
      executable: accountEntry.account.executable,
      lamports: accountEntry.account.lamports,
      owner: accountEntry.account.owner,
      rentEpoch: accountEntry.account.rentEpoch,
      programAddress: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
      space: BigInt(data.length),
    }

    if (kind === AccountDiscriminator.FixedDelegation) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const decoded = decodeFixedDelegation(encodedAccount as any)
      all.push({
        address: accountEntry.pubkey,
        type: 'Fixed',
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        data: (decoded as any).data as DelegationData,
      })
    } else if (kind === AccountDiscriminator.RecurringDelegation) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const decoded = decodeRecurringDelegation(encodedAccount as any)
      all.push({
        address: accountEntry.pubkey,
        type: 'Recurring',
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        data: (decoded as any).data as DelegationData,
      })
    }
  }

  return {
    fixed: all.filter((d) => d.type === 'Fixed'),
    recurring: all.filter((d) => d.type === 'Recurring'),
    all,
  }
}

function useDelegationsByRole(role: DelegationRole, _tokenMint?: string | null) {
  const { account, cluster } = useWalletUi()
  const clusterConfig = cluster as unknown as ClusterWithUrl
  const queryClient = useQueryClient()

  const query = useQuery({
    queryKey: ['delegations', role, account?.address, cluster.id],
    queryFn: async (): Promise<GroupedDelegations> => {
      if (!account?.address) {
        return { fixed: [], recurring: [], all: [] }
      }
      return fetchDelegationsByRole(clusterConfig.url, account.address, role)
    },
    enabled: !!account?.address,
  })

  const refetch = async () => {
    await queryClient.invalidateQueries({
      queryKey: ['delegations', role, account?.address, cluster.id],
    })
    await query.refetch()
  }

  return {
    ...query,
    refetch,
    fixed: query.data?.fixed ?? [],
    recurring: query.data?.recurring ?? [],
    all: query.data?.all ?? [],
    isEmpty: (query.data?.all.length ?? 0) === 0,
  }
}

/**
 * Hook to fetch delegations where the connected wallet is the DELEGATOR.
 * These are delegations the user has created (outgoing).
 */
export function useDelegations(tokenMint?: string | null) {
  return useDelegationsByRole('delegator', tokenMint)
}

/**
 * Hook to fetch delegations where the connected wallet is the DELEGATEE.
 * These are delegations others have created for the user (incoming).
 */
export function useIncomingDelegations(tokenMint?: string | null) {
  return useDelegationsByRole('delegatee', tokenMint)
}
