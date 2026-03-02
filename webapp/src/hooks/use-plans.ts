import { useQuery } from '@tanstack/react-query'
import { useWalletUi } from '@wallet-ui/react'
import { createSolanaRpc, address } from 'gill'
import {
  PLAN_SIZE,
  PLAN_OWNER_OFFSET,
  decodePlan,
} from '@multidelegator/client'
import type { PlanData } from '@multidelegator/client'
import { useClusterConfig } from '@/hooks/use-cluster-config'
import { useProgramAddress } from '@/hooks/use-token-config'
import type { DelegationAccountRaw } from '@/lib/types'
import { decodeBase64ToUint8Array } from '@/lib/utils'

export interface PlanItem {
  address: string
  owner: string
  status: number
  data: PlanData
}

async function fetchPlansByMerchant(rpcUrl: string, merchantAddress: string, progAddr: string): Promise<PlanItem[]> {
  const rpc = createSolanaRpc(rpcUrl)

  const response = await rpc
    .getProgramAccounts(address(progAddr), {
      filters: [
        { dataSize: BigInt(PLAN_SIZE) },
        {
          memcmp: {
            offset: BigInt(PLAN_OWNER_OFFSET),
            bytes: merchantAddress,
            encoding: 'base58',
          },
        },
      ],
      encoding: 'base64',
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any)
    .send()

  const accounts = response as unknown as DelegationAccountRaw[]
  const plans: PlanItem[] = []

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
        programAddress: address(progAddr),
        space: BigInt(data.length),
      }

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const decoded = decodePlan(encodedAccount as any)
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const plan = (decoded as any).data

      plans.push({
        address: entry.pubkey,
        owner: plan.owner,
        status: plan.status,
        data: plan.data,
      })
    } catch {
      console.warn('Failed to decode plan account:', entry.pubkey)
    }
  }

  return plans
}

export function useMerchantPlans(merchantAddress: string | null) {
  const clusterConfig = useClusterConfig()
  const progAddr = useProgramAddress()

  return useQuery({
    queryKey: ['plans', merchantAddress, clusterConfig.id],
    queryFn: () => fetchPlansByMerchant(clusterConfig.url, merchantAddress!, progAddr!),
    enabled: !!merchantAddress && merchantAddress.length > 30 && !!progAddr,
  })
}

export function useMyPlans() {
  const { account } = useWalletUi()
  const clusterConfig = useClusterConfig()
  const progAddr = useProgramAddress()

  return useQuery({
    queryKey: ['plans', 'my', account?.address, clusterConfig.id],
    queryFn: () => fetchPlansByMerchant(clusterConfig.url, account!.address, progAddr!),
    enabled: !!account?.address && !!progAddr,
  })
}
