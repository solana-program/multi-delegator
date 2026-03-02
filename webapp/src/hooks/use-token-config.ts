import { useQuery } from '@tanstack/react-query'
import { api, clusterIdToNetwork } from '@/lib/api-client'
import type { NetworkConfigResponse } from '@/lib/api-client'
import { useClusterConfig } from '@/hooks/use-cluster-config'

export function useNetworkConfig() {
  const { id } = useClusterConfig()
  const network = clusterIdToNetwork(id)

  return useQuery<NetworkConfigResponse>({
    queryKey: ['network-config', network],
    queryFn: () => api.config.getNetworkConfig(network),
    staleTime: 30_000,
    retry: 2,
  })
}

export function useTokenConfig() {
  const { data, ...rest } = useNetworkConfig()
  return { data: data?.tokens, ...rest }
}

export function useProgramAddress(): string | null {
  const { data } = useNetworkConfig()
  return data?.programAddress ?? null
}

export function useUsdcMintRaw() {
  const { data: tokens, isLoading } = useTokenConfig()
  return {
    mint: tokens?.find((t) => t.symbol === 'USDC')?.mint ?? null,
    isLoading
  }
}

export function useUsdcMint(): string | null {
  const { data: tokens } = useTokenConfig()
  return tokens?.find((t) => t.symbol === 'USDC')?.mint ?? null
}

export function useUsdcConfig() {
  const { data: tokens, ...rest } = useTokenConfig()
  const usdc = tokens?.find((t) => t.symbol === 'USDC') ?? null
  return { data: usdc, ...rest }
}
