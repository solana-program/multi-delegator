import { useQuery } from '@tanstack/react-query'
import { useClusterConfig } from '@/hooks/use-cluster-config'
import { api } from '@/lib/api-client'
import { useProgramAddress } from '@/hooks/use-token-config'

export function useProgramStatus() {
  const { url, id } = useClusterConfig()
  const progAddr = useProgramAddress()
  return useQuery({
    queryKey: ['program-status', id, progAddr],
    queryFn: () => api.program.status(progAddr!, url),
    enabled: id !== 'solana:localnet' && !!progAddr,
    staleTime: 30_000,
  })
}

export function useBinaryInfo() {
  return useQuery({
    queryKey: ['binary-info'],
    queryFn: () => api.program.binaryInfo(),
    staleTime: 60_000,
  })
}
