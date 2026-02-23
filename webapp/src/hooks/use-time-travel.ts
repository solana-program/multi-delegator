import { useCallback } from 'react'
import { useClusterConfig } from '@/hooks/use-cluster-config'

interface RpcResponse<T = unknown> {
  jsonrpc: string
  id: number
  result?: T
  error?: { code: number; message: string }
}

export interface ClockState {
  absoluteSlot: number
  blockHeight: number
  epoch: number
  slotIndex: number
  slotsInEpoch: number
  transactionCount: number
}

async function rpcCall<T = unknown>(url: string, method: string, params: unknown = []): Promise<T> {
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
  })
  if (!res.ok) throw new Error(`RPC request failed: ${res.status}`)
  const data: RpcResponse<T> = await res.json()
  if (data.error) throw new Error(data.error.message)
  return data.result as T
}

const CLOCK_SYSVAR = 'SysvarC1ock11111111111111111111111111111111'

export async function getBlockTimestamp(url: string): Promise<number> {
  const res = await rpcCall<{ value: { data: [string, string] } }>(
    url, 'getAccountInfo', [CLOCK_SYSVAR, { encoding: 'base64' }],
  )
  const raw = atob(res.value.data[0])
  const bytes = new Uint8Array(raw.length)
  for (let i = 0; i < raw.length; i++) bytes[i] = raw.charCodeAt(i)
  const view = new DataView(bytes.buffer)
  return Number(view.getBigInt64(32, true))
}

export function useTimeTravel() {
  const { url } = useClusterConfig()

  const timeTravel = useCallback(
    (timestampSec: number) => rpcCall<ClockState>(url, 'surfnet_timeTravel', [{ absoluteTimestamp: timestampSec * 1000 }]),
    [url],
  )

  const pauseClock = useCallback(
    () => rpcCall<ClockState>(url, 'surfnet_pauseClock', []),
    [url],
  )

  const resumeClock = useCallback(
    () => rpcCall<ClockState>(url, 'surfnet_resumeClock', []),
    [url],
  )

  const getCurrentTimestamp = useCallback(
    () => getBlockTimestamp(url),
    [url],
  )

  return { timeTravel, pauseClock, resumeClock, getCurrentTimestamp }
}
