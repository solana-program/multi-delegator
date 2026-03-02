import type { ReactNode } from 'react'
import {
  createSolanaDevnet,
  createSolanaLocalnet,
  createSolanaTestnet,
  createWalletUiConfig,
  WalletUi,
  WalletUiClusterDropdown,
  WalletUiDropdown,
} from '@wallet-ui/react'
import { WalletSignerProvider } from './use-wallet-ui-signer'

export { WalletUiDropdown as WalletButton, WalletUiClusterDropdown as ClusterButton }

const defaultClusterId = import.meta.env.VITE_DEFAULT_CLUSTER ?? 'solana:localnet'

const allClusters = [
  createSolanaDevnet(),
  createSolanaTestnet(),
  createSolanaLocalnet(),
]

const config = createWalletUiConfig({
  clusters: allClusters.sort((a, b) =>
    a.id === defaultClusterId ? -1 : b.id === defaultClusterId ? 1 : 0
  ),
})

export function SolanaProvider({ children }: { children: ReactNode }) {
  return (
    <WalletUi config={config}>
      <WalletSignerProvider>{children}</WalletSignerProvider>
    </WalletUi>
  )
}
