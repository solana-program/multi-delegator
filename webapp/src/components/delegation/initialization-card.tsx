import { useWalletUi } from '@wallet-ui/react'
import { address } from 'gill'
import { Shield, Check, AlertCircle } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useGetTokenAccountsQuery } from '@/components/account/account-data-access'
import { useMultiDelegatorMutations } from '@/hooks/use-multi-delegator'
import { useQueryClient } from '@tanstack/react-query'
import type { TokenAccountEntry } from '@/lib/types'
import { invalidateWithDelay } from '@/lib/utils'

interface InitializationCardProps {
  tokenMint: string
  onSuccess?: () => void
}

/**
 * Card component shown when the MultiDelegate PDA is not initialized.
 * Explains what initialization does and provides a button to initialize.
 */
export function InitializationCard({ tokenMint, onSuccess }: InitializationCardProps) {
  const { account } = useWalletUi()
  const { initMultiDelegate } = useMultiDelegatorMutations()
  const queryClient = useQueryClient()

  // Get user's token accounts to find the ATA for this mint
  const { data: tokenAccounts, isLoading: tokenAccountsLoading } = useGetTokenAccountsQuery({
    address: address(account?.address ?? ''),
  })

  // Find the ATA for the specified token mint
  const userAta = (tokenAccounts as TokenAccountEntry[] | undefined)?.find((entry) => {
    return entry.account?.data?.parsed?.info?.mint === tokenMint
  })

  const userAtaAddress = userAta?.pubkey ?? null
  // Get the token program from the account owner (Token vs Token2022)
  const tokenProgram = userAta?.account?.owner ?? null

  const handleInitialize = async () => {
    if (!userAtaAddress || !tokenProgram) return

    await initMultiDelegate.mutateAsync(
      { tokenMint, userAta: userAtaAddress, tokenProgram },
      {
        onSuccess: () => {
          invalidateWithDelay(queryClient, [['multiDelegateStatus'], ['get-token-accounts']])
          onSuccess?.()
        },
      }
    )
  }

  const isPending = initMultiDelegate.isPending
  const hasAta = !!userAtaAddress
  const isLoading = tokenAccountsLoading

  return (
    <Card className="relative overflow-hidden border-amber-500/30 bg-gradient-to-br from-amber-950/30 via-amber-900/10 to-transparent">
      <CardHeader className="pb-2">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-full bg-amber-500/20">
            <Shield className="h-6 w-6 text-amber-400" />
          </div>
          <CardTitle className="text-lg">Enable Token Delegations</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Initialize your delegation account to start creating delegations for this token. This is a one-time setup.
        </p>

        <ul className="space-y-2 text-sm">
          <li className="flex items-center gap-2 text-muted-foreground">
            <Check className="h-4 w-4 text-green-500" />
            One-time setup per token
          </li>
          <li className="flex items-center gap-2 text-muted-foreground">
            <Check className="h-4 w-4 text-green-500" />
            Enables fixed and recurring delegations
          </li>
          <li className="flex items-center gap-2 text-muted-foreground">
            <Check className="h-4 w-4 text-green-500" />
            Approves the delegation program to manage transfers
          </li>
        </ul>

        {!hasAta && !isLoading && (
          <div className="flex items-center gap-2 p-3 rounded-md bg-destructive/10 border border-destructive/20">
            <AlertCircle className="h-4 w-4 text-destructive" />
            <p className="text-sm text-destructive">
              No token account found. Please get some USDC first using the faucet.
            </p>
          </div>
        )}

        <Button
          onClick={handleInitialize}
          disabled={isPending || !hasAta || isLoading}
          className="w-full"
          size="lg"
        >
          {isPending ? (
            'Initializing...'
          ) : isLoading ? (
            'Loading...'
          ) : !hasAta ? (
            'Token Account Required'
          ) : (
            'Initialize Delegation Account'
          )}
        </Button>
      </CardContent>
    </Card>
  )
}
