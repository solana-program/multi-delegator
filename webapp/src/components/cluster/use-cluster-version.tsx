import { useWalletUi } from '@wallet-ui/react'
import { useQuery } from '@tanstack/react-query'
import { createSolanaRpc } from 'gill'

interface ClusterWithUrl {
  url: string
  id: string
  label: string
}

export function useClusterVersion() {
  const { cluster } = useWalletUi()
  const clusterConfig = cluster as unknown as ClusterWithUrl
  const rpc = createSolanaRpc(clusterConfig.url)
  
  return useQuery({
    retry: false,
    queryKey: ['version', { cluster: clusterConfig.id }],
    queryFn: () => rpc.getVersion().send(),
  })
}
