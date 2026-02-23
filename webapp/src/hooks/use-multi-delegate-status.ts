import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useWalletUi } from '@wallet-ui/react'
import { createSolanaRpc, address, type Address } from 'gill'
import { TOKEN_PROGRAM_ADDRESS, TOKEN_2022_PROGRAM_ADDRESS } from 'gill/programs/token'
import { getMultiDelegatePDA, fetchMaybeMultiDelegate } from '@multidelegator/client'
import { useClusterConfig } from '@/hooks/use-cluster-config'
import type { TokenAccountEntry } from '@/lib/types'

const TOKEN_PROGRAMS: Address[] = [TOKEN_PROGRAM_ADDRESS, TOKEN_2022_PROGRAM_ADDRESS]

export interface MultiDelegateData {
  owner: string
  tokenMint: string
  bump: number
}

export interface MultiDelegateStatus {
  initialized: boolean
  approved: boolean
  pda: string | null
  data: MultiDelegateData | null
}

/**
 * Hook to check if the MultiDelegate PDA is initialized for the connected wallet and token mint.
 * The MultiDelegate must be initialized before creating any delegations.
 * Initialization also sets up SPL token delegation to the PDA.
 *
 * @param tokenMint - The token mint address to check initialization for
 */
export function useMultiDelegateStatus(tokenMint: string | null) {
  const { account, cluster } = useWalletUi()
  const clusterConfig = useClusterConfig()
  const queryClient = useQueryClient()

  const query = useQuery({
    queryKey: ['multiDelegateStatus', account?.address, tokenMint, cluster.id],
    queryFn: async (): Promise<MultiDelegateStatus> => {
      if (!account?.address || !tokenMint) {
        return { initialized: false, approved: false, pda: null, data: null }
      }

      const rpc = createSolanaRpc(clusterConfig.url)
      const [pda] = await getMultiDelegatePDA(address(account.address), address(tokenMint))
      const multiDelegate = await fetchMaybeMultiDelegate(rpc, pda)

      const exists = multiDelegate && 'exists' in multiDelegate ? multiDelegate.exists : false

      let approved = false
      if (exists) {
        try {
          const tokenAccounts = await Promise.all(
            TOKEN_PROGRAMS.map((programId) =>
              rpc
                .getTokenAccountsByOwner(address(account.address), { programId }, { encoding: 'jsonParsed', commitment: 'confirmed' })
                .send()
                .then((res) => (res.value ?? []) as unknown as TokenAccountEntry[])
                .catch(() => [] as TokenAccountEntry[])
            )
          ).then((results) => results.flat())

          const matchingAccount = tokenAccounts.find((entry) => {
            const info = entry.account?.data?.parsed?.info
            return info?.mint === tokenMint
          })

          const delegate = matchingAccount?.account?.data?.parsed?.info?.delegate ?? null
          approved = delegate === pda
        } catch (err) {
          console.error('Failed to fetch token accounts:', err)
        }
      }

      return {
        initialized: exists,
        approved,
        pda: pda,
        data: exists && multiDelegate && 'data' in multiDelegate ? multiDelegate.data as unknown as MultiDelegateData : null,
      }
    },
    enabled: !!account?.address && !!tokenMint,
    retry: 2,
    retryDelay: (attempt) => Math.min(1000 * 2 ** attempt, 5000),
  })

  const refetch = async () => {
    await queryClient.invalidateQueries({
      queryKey: ['multiDelegateStatus', account?.address, tokenMint, cluster.id],
    })
  }

  return {
    ...query,
    refetch,
    isInitialized: query.data?.initialized ?? false,
    isApproved: query.data?.approved ?? false,
    pda: query.data?.pda ?? null,
  }
}
