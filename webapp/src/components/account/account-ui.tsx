import { useWalletUi } from '@wallet-ui/react'
import { address, lamportsToSol } from 'gill'
import type { Address, Lamports } from 'gill'
import { Link } from 'react-router'
import { Button } from '@/components/ui/button'
import { AppAlert } from '@/components/app-alert'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { useQueryClient } from '@tanstack/react-query'
import { useMemo, useState } from 'react'
import { RefreshCw, Wallet, DollarSign, Coins } from 'lucide-react'
import { toast } from 'sonner'
import type { TokenAccountEntry } from '@/lib/types'
import {
  useGetBalanceQuery,
  useGetTokenAccountsQuery,
  useAirdropSol,
  useAirdropUsdc,
} from './account-data-access'

export function AccountChecker() {
  const { account } = useWalletUi()
  if (!account) {
    return null
  }
  return <AccountBalanceCheck address={address(account.address)} />
}

export function AccountBalanceCheck({ address: addr }: { address: Address }) {
  const { cluster } = useWalletUi()
  const query = useGetBalanceQuery({ address: addr })

  if (query.isLoading) {
    return null
  }
  if (query.isError || !query.data?.value) {
    return (
      <AppAlert
        action={
          <Button variant="outline" asChild>
            <Link to="/faucet">Request Airdrop</Link>
          </Button>
        }
      >
        You are connected to <strong>{cluster.label}</strong> but your account is not found on this cluster.
      </AppAlert>
    )
  }
  return null
}

export function AccountBalance({ address: addr }: { address: Address }) {
  const query = useGetBalanceQuery({ address: addr })

  return (
    <h1 className="text-5xl font-bold cursor-pointer" onClick={() => query.refetch()}>
      {query.data?.value ? <BalanceSol balance={query.data?.value} /> : '...'} SOL
    </h1>
  )
}

export function WalletBalanceCards({ address: addr }: { address: Address }) {
  const solQuery = useGetBalanceQuery({ address: addr })
  const tokenQuery = useGetTokenAccountsQuery({ address: addr })

  const usdcAccount = useMemo(() => {
    return (tokenQuery.data as TokenAccountEntry[] | undefined)?.find((entry) => {
      return entry.account?.data?.parsed?.info?.tokenAmount?.uiAmount > 0
    })
  }, [tokenQuery.data])

  const usdcBalance = usdcAccount?.account?.data?.parsed?.info?.tokenAmount?.uiAmount ?? 0

  const handleRefresh = async () => {
    await Promise.all([solQuery.refetch(), tokenQuery.refetch()])
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Wallet Overview</h2>
        <Button
          variant="outline"
          size="sm"
          onClick={handleRefresh}
          disabled={solQuery.isLoading || tokenQuery.isLoading}
        >
          <RefreshCw className={`h-4 w-4 ${solQuery.isLoading || tokenQuery.isLoading ? 'animate-spin' : ''}`} />
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card className="relative overflow-hidden border border-purple-500/20 bg-gradient-to-br from-purple-950/40 via-purple-900/20 to-transparent hover:border-purple-500/40 transition-all duration-300">
          <CardHeader className="relative pb-2">
            <CardTitle className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-400">Solana Balance</span>
              <Wallet className="h-5 w-5 text-purple-400" />
            </CardTitle>
          </CardHeader>
          <CardContent className="relative pt-2">
            <div className="space-y-1">
              <div className="text-5xl font-bold tracking-tight">
                {solQuery.data?.value ? (
                  <span className="text-purple-400">{Number(lamportsToSol(solQuery.data.value)).toFixed(4)}</span>
                ) : (
                  <span className="text-muted-foreground">...</span>
                )}
              </div>
              <div className="text-sm font-medium text-gray-500">SOL</div>
            </div>
          </CardContent>
        </Card>

        <Card className="relative overflow-hidden border border-green-500/20 bg-gradient-to-br from-green-950/40 via-green-900/20 to-transparent hover:border-green-500/40 transition-all duration-300">
          <CardHeader className="relative pb-2">
            <CardTitle className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-400">USDC Balance</span>
              <DollarSign className="h-5 w-5 text-green-400" />
            </CardTitle>
          </CardHeader>
          <CardContent className="relative pt-2">
            <div className="space-y-1">
              <div className="text-5xl font-bold tracking-tight">
                {tokenQuery.isLoading ? (
                  <span className="text-muted-foreground">...</span>
                ) : (
                  <span className="text-green-400">
                    {usdcBalance.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                  </span>
                )}
              </div>
              <div className="text-sm font-medium text-gray-500">USDC</div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}

export function DevFaucet() {
  const { account, cluster } = useWalletUi()
  const queryClient = useQueryClient()
  const [solAmount, setSolAmount] = useState('1')
  const [usdcAmount, setUsdcAmount] = useState('1000')

  const airdropSol = useAirdropSol()
  const airdropUsdc = useAirdropUsdc()

  const handleRefreshTokenAccounts = async () => {
    await queryClient.invalidateQueries({ queryKey: ['get-token-accounts'] })
    await queryClient.invalidateQueries({ queryKey: ['get-balance'] })
    toast.success('Balances refreshed!')
  }

  if (!account || cluster.id === 'solana:mainnet') {
    return null
  }

  const handleAirdropSol = async () => {
    if (!solAmount || parseFloat(solAmount) <= 0) {
      toast.error('Please enter a valid SOL amount')
      return
    }
    await airdropSol.mutateAsync(parseFloat(solAmount))
  }

  const handleAirdropUsdc = async () => {
    if (!usdcAmount || parseFloat(usdcAmount) <= 0) {
      toast.error('Please enter a valid USDC amount')
      return
    }
    await airdropUsdc.mutateAsync(parseFloat(usdcAmount))
  }

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2">
          <Coins className="h-5 w-5 text-blue-500" />
          Development Faucet
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="flex gap-2 items-center">
            <Input
              type="number"
              placeholder="SOL"
              value={solAmount}
              onChange={(e) => setSolAmount(e.target.value)}
              min="0.1"
              step="0.1"
              className="w-24"
            />
            <Button onClick={handleAirdropSol} disabled={airdropSol.isPending} size="sm" variant="outline">
              <Coins className="h-4 w-4 mr-1" />
              Get SOL
            </Button>
          </div>

          <div className="flex gap-2 items-center">
            <Input
              type="number"
              placeholder="USDC"
              value={usdcAmount}
              onChange={(e) => setUsdcAmount(e.target.value)}
              min="1"
              step="100"
              className="w-24"
            />
            <Button onClick={handleAirdropUsdc} disabled={airdropUsdc.isPending} size="sm" variant="outline">
              <DollarSign className="h-4 w-4 mr-1" />
              Get USDC
            </Button>
          </div>

          <Button onClick={handleRefreshTokenAccounts} size="sm" variant="ghost">
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

function BalanceSol({ balance }: { balance: Lamports }) {
  return <span>{lamportsToSol(balance)}</span>
}
