import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api-client'

/**
 * Hook to fetch token configuration from the API.
 * Tokens are configured server-side and include mint addresses, decimals, etc.
 */
export function useTokenConfig() {
  return useQuery({
    queryKey: ['token-config'],
    queryFn: () => api.config.getTokens(),
    staleTime: Infinity, // Config rarely changes during a session
    retry: 2,
  })
}

/**
 * Hook to get the USDC mint address from the token configuration.
 * Returns null if USDC is not configured or config is still loading.
 */
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

/**
 * Hook to get the USDC token config including decimals and other metadata.
 */
export function useUsdcConfig() {
  const { data: tokens, ...rest } = useTokenConfig()
  const usdc = tokens?.find((t) => t.symbol === 'USDC') ?? null
  return { data: usdc, ...rest }
}
