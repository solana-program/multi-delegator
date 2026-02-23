import { useMemo } from 'react'
import { CalendarCheck } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { useMySubscriptions, type EnrichedSubscription } from '@/hooks/use-subscriptions'
import { USDC_MULTIPLIER, ellipsify } from '@/lib/utils'

function formatPeriod(hours: bigint): string {
  const h = Number(hours)
  if (h === 24) return 'Daily'
  if (h === 168) return 'Weekly'
  if (h === 720) return 'Monthly'
  if (h === 8760) return 'Yearly'
  if (h > 24 && h % 24 === 0) return `Every ${h / 24} days`
  return `Every ${h} hours`
}

function parsePlanMeta(uri: string): { n?: string; d?: string } {
  try { return JSON.parse(uri) } catch { return {} }
}

function SubscriptionRow({ item }: { item: EnrichedSubscription }) {
  const isActive = Number(item.subscription.revokedTs) === 0
  const meta = useMemo(() => item.plan ? parsePlanMeta(item.plan.data.metadataUri) : {}, [item.plan])
  const planName = meta.n || 'Unknown Plan'
  const amount = item.plan ? Number(item.plan.data.amount) / USDC_MULTIPLIER : null
  const period = item.plan ? formatPeriod(item.plan.data.periodHours) : null
  const pulled = Number(item.subscription.amountPulledInPeriod) / USDC_MULTIPLIER

  return (
    <div className="flex items-center justify-between gap-4 rounded-lg border border-amber-500/10 bg-amber-950/20 px-4 py-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="font-medium truncate">{planName}</p>
          {isActive ? (
            <Badge variant="outline" className="text-emerald-400 border-emerald-400/30 text-xs">Active</Badge>
          ) : (
            <Badge variant="outline" className="text-red-400 border-red-400/30 text-xs">Cancelled</Badge>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground mt-1">
          <span className="font-mono">{ellipsify(item.address, 4)}</span>
          {amount !== null && period && (
            <>
              <span className="text-border">|</span>
              <span>${amount} / {period.toLowerCase()}</span>
            </>
          )}
          <span className="text-border">|</span>
          <span>Pulled: ${pulled.toFixed(2)}</span>
        </div>
      </div>
    </div>
  )
}

export function MySubscriptionsPanel() {
  const { data: subscriptions, isLoading } = useMySubscriptions()

  if (isLoading) {
    return (
      <Card className="border-amber-500/20 bg-gradient-to-br from-amber-950/40 via-amber-900/20 to-transparent">
        <CardContent className="flex items-center justify-center py-12">
          <div className="animate-pulse text-muted-foreground">Loading subscriptions...</div>
        </CardContent>
      </Card>
    )
  }

  const hasSubs = subscriptions && subscriptions.length > 0

  return (
    <Card className="relative overflow-hidden border-amber-500/20 bg-gradient-to-br from-amber-950/40 via-amber-900/20 to-transparent hover:border-amber-500/40 transition-all duration-300">
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <CalendarCheck className="h-5 w-5 text-amber-400" />
            <CardTitle>My Subscriptions</CardTitle>
          </div>
          {hasSubs && (
            <Badge variant="outline" className="text-amber-400 border-amber-400/30">
              {subscriptions.length}
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent>
          {hasSubs ? (
            <div className="space-y-2">
              {subscriptions.map((item) => (
                <SubscriptionRow key={item.address} item={item} />
              ))}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center py-8 gap-2 text-muted-foreground">
              <CalendarCheck className="h-8 w-8" />
              <p className="text-sm">No subscriptions yet</p>
              <p className="text-xs">Subscribe to plans from the Marketplace</p>
            </div>
          )}
      </CardContent>
    </Card>
  )
}
