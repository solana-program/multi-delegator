import { useWalletUi } from '@wallet-ui/react'
import { address } from 'gill'
import { WalletBalanceCards } from '../components/account/account-ui'
import { DelegationManagementPanel } from '../components/delegation/delegation-management-panel'

function DashboardConnected() {
  const { account } = useWalletUi()

  return (
    <div className="space-y-8">
      {account && <WalletBalanceCards address={address(account.address)} />}
      <DelegationManagementPanel />
    </div>
  )
}

export function Dashboard() {
  const { account } = useWalletUi()

  if (!account) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[50vh] gap-4">
        <h1 className="text-2xl font-bold">Connect your wallet to get started</h1>
        <p className="text-muted-foreground">Manage your Solana delegations securely.</p>
      </div>
    )
  }

  return <DashboardConnected />
}
