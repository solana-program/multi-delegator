const API_BASE_URL = import.meta.env?.VITE_API_URL ?? 'http://localhost:3001'

export class ApiError extends Error {
  status?: number
  details?: unknown

  constructor(message: string, status?: number, details?: unknown) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.details = details
  }
}

export async function apiClient<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      throw new ApiError(errorData.error ?? 'API request failed', response.status, errorData)
    }

    return await response.json()
  } catch (error) {
    if (error instanceof ApiError) throw error
    throw new ApiError('Network request failed', undefined, error)
  }
}

export interface AirdropResponse {
  success: boolean
  message: string
  recipient: string
  amount: number
  mint?: string
}

export interface TokenConfig {
  symbol: string
  name: string
  mint: string
  decimals: number
}

export const api = {
  config: {
    getAll: () => apiClient<{ network: string; tokens: TokenConfig[] }>('/api/config'),
    getTokens: () => apiClient<TokenConfig[]>('/api/tokens'),
  },
  airdrop: {
    sol: (params: { recipient: string; amount: number }) =>
      apiClient<AirdropResponse>('/api/airdrop/sol', {
        method: 'POST',
        body: JSON.stringify(params),
      }),
    usdc: (params: { recipient: string; amount: number }) =>
      apiClient<AirdropResponse>('/api/airdrop/usdc', {
        method: 'POST',
        body: JSON.stringify(params),
      }),
  },
}
