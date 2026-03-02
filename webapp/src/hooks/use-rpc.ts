import { useMemo } from 'react'
import { createSolanaRpc } from 'gill'
import { useClusterConfig } from '@/hooks/use-cluster-config'

export function useRpc() {
  const { url } = useClusterConfig()
  return useMemo(() => createSolanaRpc(url), [url])
}
