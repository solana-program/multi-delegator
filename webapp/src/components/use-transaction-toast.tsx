import { toast } from 'sonner'
import { ExplorerLink } from '@/components/cluster/cluster-ui'

export function useTransactionToast() {
  return {
    onSuccess: (signature: string) => {
      toast.success('Transaction confirmed', {
        description: <ExplorerLink transaction={signature} label="View Transaction" />,
      })
    },
    onError: (error: Error) => {
      toast.error('Transaction failed', {
        description: error.message,
      })
    },
  }
}
