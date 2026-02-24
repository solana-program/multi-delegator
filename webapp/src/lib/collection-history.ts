const STORAGE_KEY = 'collect-payments-history'
const MAX_RECORDS = 50

export interface CollectionRecord {
  id: string
  timestamp: number
  planAddress: string
  planName: string
  subscribersCollected: number
  subscribersTotal: number
  amountPerSubscriber: number
  status: 'success' | 'partial' | 'failed'
  signatures: string[]
  error?: string
}

export function getCollectionHistory(planAddress?: string): CollectionRecord[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const all: CollectionRecord[] = JSON.parse(raw)
    return planAddress ? all.filter((r) => r.planAddress === planAddress) : all
  } catch (err) {
    console.error('Failed to parse collection history:', err)
    return []
  }
}

export function clearCollectionHistory(): void {
  localStorage.removeItem(STORAGE_KEY)
}

export function addCollectionRecord(record: CollectionRecord): void {
  const existing = getCollectionHistory()
  const updated = [record, ...existing].slice(0, MAX_RECORDS)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(updated))
}

export function createSuccessRecord(
  planAddress: string,
  planName: string,
  res: { collected: number; partial: boolean; signatures: string[] },
  subscribersTotal: number,
  amountPerSubscriber: number,
): CollectionRecord {
  return {
    id: crypto.randomUUID(),
    timestamp: Math.floor(Date.now() / 1000),
    planAddress,
    planName,
    subscribersCollected: res.collected,
    subscribersTotal,
    amountPerSubscriber,
    status: res.partial ? 'partial' : 'success',
    signatures: res.signatures,
  }
}

export function createFailureRecord(
  planAddress: string,
  planName: string,
  subscribersTotal: number,
  amountPerSubscriber: number,
  error: unknown,
): CollectionRecord {
  return {
    id: crypto.randomUUID(),
    timestamp: Math.floor(Date.now() / 1000),
    planAddress,
    planName,
    subscribersCollected: 0,
    subscribersTotal,
    amountPerSubscriber,
    status: 'failed',
    signatures: [],
    error: error instanceof Error ? error.message : 'Unknown error',
  }
}
