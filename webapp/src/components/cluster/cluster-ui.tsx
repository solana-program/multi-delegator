import type { ReactNode } from 'react'
import { getExplorerLink } from 'gill'
import type { GetExplorerLinkArgs, SolanaClusterMoniker } from 'gill'
import { Button } from '@/components/ui/button'
import { AppAlert } from '@/components/app-alert'
import { useWalletUi } from '@wallet-ui/react'
import { useClusterVersion } from './use-cluster-version'

export function ExplorerLink({
  className,
  label = '',
  ...link
}: GetExplorerLinkArgs & {
  className?: string
  label: string
}) {
  const { cluster } = useWalletUi()
  // Extract cluster moniker from cluster id (e.g., "solana:devnet" -> "devnet")
  const clusterMoniker = cluster.id.split(':')[1] as SolanaClusterMoniker
  return (
    <a
      href={getExplorerLink({ ...link, cluster: clusterMoniker })}
      target="_blank"
      rel="noopener noreferrer"
      className={className ? className : `link font-mono`}
    >
      {label}
    </a>
  )
}

export function ClusterChecker({ children }: { children: ReactNode }) {
  const { cluster } = useWalletUi()
  const query = useClusterVersion()

  if (query.isLoading) {
    return null
  }

  if (query.isError || !query.data) {
    return (
      <AppAlert
        action={
          <Button variant="outline" onClick={() => query.refetch()}>
            Refresh
          </Button>
        }
        className="mb-4"
      >
        Error connecting to cluster <span className="font-bold">{cluster.label}</span>.
      </AppAlert>
    )
  }
  return children
}
