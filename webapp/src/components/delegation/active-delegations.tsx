import { RefreshCw, Coins, FileX, ArrowUpRight, ArrowDownLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { useDelegations, useIncomingDelegations, type DelegationItem } from '@/hooks/use-delegations'
import { useMultiDelegatorMutations } from '@/hooks/use-multi-delegator'
import { DELEGATION_KINDS } from '@multidelegator/client'
import { useMemo, useState } from 'react'
import { USDC_MULTIPLIER, SECONDS_PER_DAY, isExpired } from '@/lib/utils'

interface ActiveDelegationsProps {
  tokenMint: string
}

type TabType = 'outgoing' | 'incoming'
type OutgoingSubTab = 'active' | 'expired'

const ADDRESS_TRUNCATE_THRESHOLD = 12
const ADDRESS_VISIBLE_CHARS = 4

function formatAddress(addr: string): string {
  if (addr.length <= ADDRESS_TRUNCATE_THRESHOLD) return addr
  return `${addr.slice(0, ADDRESS_VISIBLE_CHARS)}...${addr.slice(-ADDRESS_VISIBLE_CHARS)}`
}

function formatExpiry(expiryTs: bigint | number): string {
  const date = new Date(Number(expiryTs) * 1000)
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: '2-digit',
  })
}

function formatAmount(amount: bigint | number): string {
  const num = Number(amount) / USDC_MULTIPLIER
  return num.toLocaleString('en-US', {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  })
}

function formatPeriod(periodS: bigint | number): string {
  const seconds = Number(periodS)
  const days = Math.floor(seconds / SECONDS_PER_DAY)
  const hours = Math.floor((seconds % SECONDS_PER_DAY) / 3600)

  if (days > 0) {
    return `${days} day${days > 1 ? 's' : ''}`
  }
  return `${hours} hour${hours > 1 ? 's' : ''}`
}

interface RevokeDelegationButtonProps {
  delegation: DelegationItem
}

function RevokeDelegationButton({ delegation }: RevokeDelegationButtonProps) {
  const [open, setOpen] = useState(false)
  const { revokeDelegation } = useMultiDelegatorMutations()

  const handleRevoke = async () => {
    await revokeDelegation.mutateAsync({
      delegationAccount: delegation.address,
    })
    setOpen(false)
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm" className="text-destructive hover:text-destructive hover:bg-destructive/10">
          Revoke
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Revoke Delegation</DialogTitle>
          <DialogDescription>
            Are you sure you want to revoke this delegation? This action cannot be undone.
          </DialogDescription>
        </DialogHeader>
        <div className="py-4">
          <p className="text-sm text-muted-foreground">
            Delegatee: <span className="font-mono">{formatAddress(delegation.data.header.delegatee)}</span>
          </p>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={handleRevoke}
            disabled={revokeDelegation.isPending}
          >
            {revokeDelegation.isPending ? 'Revoking...' : 'Revoke'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface TransferDelegationButtonProps {
  delegation: DelegationItem
  tokenMint: string
  disabled?: boolean
}

function TransferDelegationButton({ delegation, tokenMint, disabled }: TransferDelegationButtonProps) {
  const [open, setOpen] = useState(false)
  const [amount, setAmount] = useState('')
  const { transferFixed, transferRecurring } = useMultiDelegatorMutations()

  const isFixed = delegation.type === 'Fixed'
  const availableRaw = isFixed
    ? delegation.data.amount
    : delegation.data.amountPerPeriod - (delegation.data.amountPulledInPeriod ?? 0n)
  const availableAmount = formatAmount(availableRaw)

  const handleTransfer = async () => {
    const amountBigInt = BigInt(Math.floor(parseFloat(amount) * USDC_MULTIPLIER))
    const mutation = isFixed ? transferFixed : transferRecurring

    await mutation.mutateAsync({
      tokenMint,
      delegationAccount: delegation.address,
      delegator: delegation.data.header.delegator,
      amount: amountBigInt,
    })

    setOpen(false)
    setAmount('')
  }

  const isPending = transferFixed.isPending || transferRecurring.isPending

  if (disabled) {
    return (
      <Button variant="outline" size="sm" disabled className="text-muted-foreground">
        Expired
      </Button>
    )
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className="text-green-500 hover:text-green-500 hover:bg-green-500/10"
        >
          Transfer
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Transfer</DialogTitle>
          <DialogDescription>
            Enter the amount you want to transfer from this delegation.
          </DialogDescription>
        </DialogHeader>
        <div className="py-4 space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Amount (USDC)</label>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="w-full px-3 py-2 rounded-md border border-input bg-background text-sm"
            />
            <p className="text-xs text-muted-foreground">
              Available: {availableAmount} USDC
            </p>
          </div>
          <div className="text-sm text-muted-foreground">
            <p>Delegator: <span className="font-mono">{formatAddress(delegation.data.header.delegator)}</span></p>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleTransfer}
            disabled={isPending || !amount || parseFloat(amount) <= 0}
            className="bg-green-600 hover:bg-green-700"
          >
            {isPending ? 'Transferring...' : 'Transfer'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface DelegationSectionProps {
  title: string
  icon: React.ReactNode
  delegations: DelegationItem[]
  type: 'fixed' | 'recurring'
  mode: TabType
  showExpired?: boolean
  tokenMint: string
}

function DelegationSection({ title, icon, delegations, type, mode, showExpired, tokenMint }: DelegationSectionProps) {
  if (delegations.length === 0) return null

  const isOutgoing = mode === 'outgoing'
  const partyLabel = isOutgoing ? 'Delegatee' : 'Delegator'

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
        {icon}
        <span>{title}</span>
        <span className="px-1.5 py-0.5 rounded-full bg-muted text-xs">
          {delegations.length}
        </span>
      </div>
      <Card className="border-border/50">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{partyLabel}</TableHead>
              {type === 'fixed' ? (
                <TableHead>Amount</TableHead>
              ) : (
                <>
                  <TableHead>Per Period</TableHead>
                  <TableHead>Period</TableHead>
                </>
              )}
              <TableHead>Expiry</TableHead>
              <TableHead className="w-[100px]"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {delegations.map((delegation) => {
              const rowExpired = showExpired || (!isOutgoing && isExpired(delegation.data.expiryTs))
              return (
                <TableRow key={delegation.address} className={rowExpired ? 'opacity-60' : ''}>
                  <TableCell className="font-mono text-xs">
                    {formatAddress(
                      isOutgoing
                        ? delegation.data.header.delegatee
                        : delegation.data.header.delegator
                    )}
                  </TableCell>
                  {type === 'fixed' ? (
                    <TableCell>{formatAmount(delegation.data.amount)} USDC</TableCell>
                  ) : (
                    <>
                      <TableCell>{formatAmount(delegation.data.amountPerPeriod)} USDC</TableCell>
                      <TableCell>{formatPeriod(delegation.data.periodLengthS)}</TableCell>
                    </>
                  )}
                  <TableCell>
                    {rowExpired ? (
                      <span className="text-destructive font-medium">Expired</span>
                    ) : (
                      formatExpiry(delegation.data.expiryTs)
                    )}
                  </TableCell>
                  <TableCell>
                    {isOutgoing ? (
                      <RevokeDelegationButton delegation={delegation} />
                    ) : (
                      <TransferDelegationButton delegation={delegation} tokenMint={tokenMint} disabled={rowExpired} />
                    )}
                  </TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
      </Card>
    </div>
  )
}

function EmptyState({ mode, isExpiredTab }: { mode: TabType; isExpiredTab?: boolean }) {
  const isOutgoing = mode === 'outgoing'

  if (isExpiredTab) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-center">
        <FileX className="h-12 w-12 text-muted-foreground/50 mb-4" />
        <h3 className="text-lg font-medium text-muted-foreground">No expired delegations</h3>
        <p className="text-sm text-muted-foreground/70 mt-1">All your delegations are still active</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center justify-center py-12 text-center">
      <FileX className="h-12 w-12 text-muted-foreground/50 mb-4" />
      <h3 className="text-lg font-medium text-muted-foreground">
        {isOutgoing ? 'No active delegations' : 'No delegations received'}
      </h3>
      <p className="text-sm text-muted-foreground/70 mt-1">
        {isOutgoing
          ? 'Create your first delegation to get started'
          : 'No one has delegated active tokens to you yet'}
      </p>
    </div>
  )
}

interface TabButtonProps {
  active: boolean
  onClick: () => void
  icon: React.ReactNode
  label: string
  count: number
}

function TabButton({ active, onClick, icon, label, count }: TabButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
        active
          ? 'bg-primary text-primary-foreground'
          : 'text-muted-foreground hover:text-foreground hover:bg-muted'
      }`}
    >
      {icon}
      {label}
      <span
        className={`px-1.5 py-0.5 rounded-full text-xs ${
          active ? 'bg-primary-foreground/20' : 'bg-muted'
        }`}
      >
        {count}
      </span>
    </button>
  )
}

interface SubTabButtonProps {
  active: boolean
  onClick: () => void
  label: string
  count: number
  variant?: 'default' | 'warning'
}

function SubTabButton({ active, onClick, label, count, variant = 'default' }: SubTabButtonProps) {
  const isWarning = variant === 'warning' && count > 0
  return (
    <button
      onClick={onClick}
      className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
        active
          ? 'bg-muted text-foreground'
          : 'text-muted-foreground hover:text-foreground'
      }`}
    >
      {label}
      <span
        className={`ml-1.5 px-1.5 py-0.5 rounded-full text-xs ${
          isWarning
            ? 'bg-destructive/20 text-destructive'
            : active
              ? 'bg-background'
              : 'bg-muted'
        }`}
      >
        {count}
      </span>
    </button>
  )
}

export function ActiveDelegations({ tokenMint }: ActiveDelegationsProps) {
  const [activeTab, setActiveTab] = useState<TabType>('outgoing')
  const [outgoingSubTab, setOutgoingSubTab] = useState<OutgoingSubTab>('active')

  const outgoing = useDelegations(tokenMint)
  const incoming = useIncomingDelegations(tokenMint)

  const outgoingFiltered = useMemo(() => {
    const active = outgoing.all.filter((d) => !isExpired(d.data.expiryTs))
    const expired = outgoing.all.filter((d) => isExpired(d.data.expiryTs))
    return {
      active: {
        all: active,
        fixed: active.filter((d) => d.type === 'Fixed'),
        recurring: active.filter((d) => d.type === 'Recurring'),
      },
      expired: {
        all: expired,
        fixed: expired.filter((d) => d.type === 'Fixed'),
        recurring: expired.filter((d) => d.type === 'Recurring'),
      },
    }
  }, [outgoing.all])

  const incomingGrouped = useMemo(() => {
    return {
      all: incoming.all,
      fixed: incoming.all.filter((d) => d.type === 'Fixed'),
      recurring: incoming.all.filter((d) => d.type === 'Recurring'),
    }
  }, [incoming.all])

  const isLoading = activeTab === 'outgoing' ? outgoing.isLoading : incoming.isLoading

  const handleRefresh = () => {
    if (activeTab === 'outgoing') {
      outgoing.refetch()
    } else {
      incoming.refetch()
    }
  }

  const renderOutgoingContent = () => {
    const data = outgoingSubTab === 'active' ? outgoingFiltered.active : outgoingFiltered.expired
    const isEmpty = data.all.length === 0
    const showExpired = outgoingSubTab === 'expired'

    if (isEmpty) {
      return <EmptyState mode="outgoing" isExpiredTab={showExpired} />
    }

    return (
      <div className="space-y-6">
        <DelegationSection
          title={DELEGATION_KINDS.fixed.label}
          icon={<Coins className="h-4 w-4" />}
          delegations={data.fixed}
          type="fixed"
          mode="outgoing"
          showExpired={showExpired}
          tokenMint={tokenMint}
        />
        <DelegationSection
          title={DELEGATION_KINDS.recurring.label}
          icon={<RefreshCw className="h-4 w-4" />}
          delegations={data.recurring}
          type="recurring"
          mode="outgoing"
          showExpired={showExpired}
          tokenMint={tokenMint}
        />
      </div>
    )
  }

  const renderIncomingContent = () => {
    const isEmpty = incomingGrouped.all.length === 0

    if (isEmpty) {
      return <EmptyState mode="incoming" />
    }

    return (
      <div className="space-y-6">
        <DelegationSection
          title={DELEGATION_KINDS.fixed.label}
          icon={<Coins className="h-4 w-4" />}
          delegations={incomingGrouped.fixed}
          type="fixed"
          mode="incoming"
          tokenMint={tokenMint}
        />
        <DelegationSection
          title={DELEGATION_KINDS.recurring.label}
          icon={<RefreshCw className="h-4 w-4" />}
          delegations={incomingGrouped.recurring}
          type="recurring"
          mode="incoming"
          tokenMint={tokenMint}
        />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Delegations</h3>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleRefresh}
          disabled={isLoading}
        >
          <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
        </Button>
      </div>

      <div className="flex gap-2">
        <TabButton
          active={activeTab === 'outgoing'}
          onClick={() => setActiveTab('outgoing')}
          icon={<ArrowUpRight className="h-4 w-4" />}
          label="My Delegations"
          count={outgoing.all.length}
        />
        <TabButton
          active={activeTab === 'incoming'}
          onClick={() => setActiveTab('incoming')}
          icon={<ArrowDownLeft className="h-4 w-4" />}
          label="Delegated to Me"
          count={incomingGrouped.all.length}
        />
      </div>

      {activeTab === 'outgoing' && (
        <div className="flex gap-1 border-b border-border pb-2">
          <SubTabButton
            active={outgoingSubTab === 'active'}
            onClick={() => setOutgoingSubTab('active')}
            label="Active"
            count={outgoingFiltered.active.all.length}
          />
          <SubTabButton
            active={outgoingSubTab === 'expired'}
            onClick={() => setOutgoingSubTab('expired')}
            label="Expired"
            count={outgoingFiltered.expired.all.length}
            variant="warning"
          />
        </div>
      )}

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      ) : activeTab === 'outgoing' ? (
        renderOutgoingContent()
      ) : (
        renderIncomingContent()
      )}
    </div>
  )
}
