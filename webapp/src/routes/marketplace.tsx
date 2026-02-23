import { useState } from 'react'
import { Store, Search, Loader2 } from 'lucide-react'
import { useWalletUi } from '@wallet-ui/react'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { PlanCard } from '@/components/plan/plan-card'
import { useMerchantPlans } from '@/hooks/use-plans'

function MarketplaceConnected() {
  const [inputValue, setInputValue] = useState('')
  const [searchAddress, setSearchAddress] = useState<string | null>(null)

  const { data: plans, isLoading, isError } = useMerchantPlans(searchAddress)

  const handleSearch = () => {
    const trimmed = inputValue.trim()
    setSearchAddress(trimmed.length > 30 ? trimmed : null)
  }

  const hasSearched = searchAddress !== null
  const hasResults = plans && plans.length > 0

  return (
    <div className="space-y-6">
      <div className="flex gap-2">
        <Input
          value={inputValue}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setInputValue(e.target.value)}
          onKeyDown={(e: React.KeyboardEvent) => e.key === 'Enter' && handleSearch()}
          placeholder="Enter merchant wallet address"
          className="font-mono text-sm"
        />
        <Button
          onClick={handleSearch}
          disabled={isLoading}
          variant="outline"
          className="gap-2 shrink-0"
        >
          {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
          Search
        </Button>
      </div>

      {isLoading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-emerald-400" />
        </div>
      )}

      {isError && (
        <Card className="border-destructive/20">
          <CardContent className="flex flex-col items-center justify-center py-8 gap-2">
            <p className="text-destructive">Failed to fetch plans. Check the address and try again.</p>
          </CardContent>
        </Card>
      )}

      {!isLoading && hasSearched && !isError && !hasResults && (
        <Card className="border-emerald-500/20 bg-gradient-to-br from-emerald-950/40 via-emerald-900/20 to-transparent">
          <CardContent className="flex flex-col items-center justify-center py-12 gap-2">
            <Search className="h-8 w-8 text-muted-foreground" />
            <p className="text-muted-foreground">No plans found for this merchant.</p>
          </CardContent>
        </Card>
      )}

      {hasResults && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {plans.map((plan, i) => (
            <div
              key={plan.address}
              className="animate-[fadeInUp_0.3s_ease-out_both]"
              style={{ animationDelay: `${i * 80}ms` }}
            >
              <PlanCard plan={plan} variant="marketplace" />
            </div>
          ))}
        </div>
      )}

      {!hasSearched && !isLoading && (
        <Card className="border-emerald-500/20 bg-gradient-to-br from-emerald-950/40 via-emerald-900/20 to-transparent">
          <CardContent className="flex flex-col items-center justify-center py-16 gap-4">
            <Store className="h-12 w-12 text-emerald-400" />
            <h2 className="text-xl font-semibold">Subscription Plans</h2>
            <p className="text-muted-foreground text-center max-w-md">
              Search for a merchant address to browse their subscription plans.
            </p>
          </CardContent>
        </Card>
      )}

    </div>
  )
}

export function Marketplace() {
  const { account } = useWalletUi()

  if (!account) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[50vh] gap-4">
        <h1 className="text-2xl font-bold">Connect your wallet to get started</h1>
        <p className="text-muted-foreground">Create and manage subscription plans on Solana.</p>
      </div>
    )
  }

  return <MarketplaceConnected />
}
