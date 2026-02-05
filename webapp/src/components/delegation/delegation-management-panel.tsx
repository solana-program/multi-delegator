import { Users, AlertCircle, CheckCircle2, ShieldAlert } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { InitializationCard } from './initialization-card'
import { ActiveDelegations } from './active-delegations'
import { CreateDelegationDialog } from './create-delegation-dialog'
import { useUsdcMint } from '@/hooks/use-token-config'
import { useMultiDelegateStatus } from '@/hooks/use-multi-delegate-status'

function LoadingState() {
  return (
    <Card className="border-blue-500/20 bg-gradient-to-br from-blue-950/40 via-blue-900/20 to-transparent">
      <CardContent className="flex items-center justify-center py-12">
        <div className="animate-pulse text-muted-foreground">Loading delegation status...</div>
      </CardContent>
    </Card>
  )
}

function TokenConfigError() {
  return (
    <Card className="border-destructive/20">
      <CardContent className="flex items-center gap-3 py-6">
        <AlertCircle className="h-5 w-5 text-destructive" />
        <div>
          <p className="font-medium">Token Configuration Error</p>
          <p className="text-sm text-muted-foreground">
            USDC token is not configured. Please ensure the API server is running.
          </p>
        </div>
      </CardContent>
    </Card>
  )
}

function StatusError({ onRetry }: { onRetry: () => void }) {
  return (
    <Card className="border-destructive/20">
      <CardContent className="flex items-center justify-between py-6">
        <div className="flex items-center gap-3">
          <AlertCircle className="h-5 w-5 text-destructive" />
          <div>
            <p className="font-medium">Failed to load delegation status</p>
            <p className="text-sm text-muted-foreground">
              Could not connect to the network. Check your connection.
            </p>
          </div>
        </div>
        <button onClick={onRetry} className="px-3 py-1 text-sm bg-destructive/10 hover:bg-destructive/20 rounded">
          Retry
        </button>
      </CardContent>
    </Card>
  )
}

/**
 * Status indicator showing MDA approval state
 */
function ApprovalStatus({ isApproved }: { isApproved: boolean }) {
  if (isApproved) {
    return (
      <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-green-500/10 border border-green-500/20">
        <CheckCircle2 className="h-5 w-5 text-green-500" />
        <div>
          <p className="text-sm font-medium text-green-500">MDA Approved</p>
          <p className="text-xs text-green-500/70">You can create and manage delegations</p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-amber-500/10 border border-amber-500/20">
      <ShieldAlert className="h-5 w-5 text-amber-500" />
      <div>
        <p className="text-sm font-medium text-amber-500">Approval Required</p>
        <p className="text-xs text-amber-500/70">Initialize to enable delegations</p>
      </div>
    </div>
  )
}

/**
 * Main delegation management panel component.
 * Handles the full delegation lifecycle:
 * - Shows approval status (approved with green check / not approved with warning)
 * - Shows initialization card when MultiDelegate PDA is not initialized
 * - Once initialized, shows active delegations and create button
 */
export function DelegationManagementPanel() {
  const usdcMint = useUsdcMint()
  const { isLoading: statusLoading, isError, isApproved, refetch: refetchStatus } = useMultiDelegateStatus(usdcMint)

  if (!usdcMint) {
    return <TokenConfigError />
  }

  if (isError) {
    return <StatusError onRetry={refetchStatus} />
  }

  if (statusLoading) {
    return <LoadingState />
  }

  return (
    <Card className="relative overflow-hidden border-blue-500/20 bg-gradient-to-br from-blue-950/40 via-blue-900/20 to-transparent hover:border-blue-500/40 transition-all duration-300">
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Users className="h-5 w-5 text-blue-400" />
            <CardTitle>Token Delegations</CardTitle>
          </div>
          <Badge variant="outline" className="text-blue-400 border-blue-400/30">
            USDC
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Always show approval status */}
        <ApprovalStatus isApproved={isApproved} />

        {/* Show initialization card when not approved */}
        {!isApproved && (
          <InitializationCard tokenMint={usdcMint} onSuccess={refetchStatus} />
        )}

        {/* Only show delegation content when approved */}
        {isApproved && (
          <div className="space-y-6">
            <ActiveDelegations tokenMint={usdcMint} />

            <div className="flex justify-center pt-2">
              <CreateDelegationDialog tokenMint={usdcMint} disabled={!isApproved} />
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
