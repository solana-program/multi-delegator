import { LayoutDashboard, Droplets, ClipboardPen, ShoppingBag, Users, Calendar, Banknote, Code2 } from 'lucide-react'
import { type LucideIcon } from 'lucide-react'

export interface NavItem {
  label: string
  path: string
  icon: LucideIcon
  children?: NavItem[]
  clusterFilter?: string[]
}

export const NAV_ITEMS: NavItem[] = [
  { label: 'Dashboard', path: '/', icon: LayoutDashboard },
  { label: 'Marketplace', path: '/marketplace', icon: ShoppingBag },
  { label: 'Delegations', path: '/delegations', icon: Users },
  { label: 'Subscriptions', path: '/subscriptions', icon: Calendar },
  {
    label: 'Plans', path: '/plans', icon: ClipboardPen,
    children: [
      { label: 'Collect Payments', path: '/plans/collect', icon: Banknote },
    ],
  },
  { label: 'Faucet', path: '/faucet', icon: Droplets, clusterFilter: ['solana:localnet', 'solana:devnet'] },
  { label: 'Program', path: '/program', icon: Code2, clusterFilter: ['solana:devnet', 'solana:testnet'] },
]
