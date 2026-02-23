import { LayoutDashboard, Droplets, ClipboardPen, ShoppingBag, Users, Calendar } from 'lucide-react'
import { type LucideIcon } from 'lucide-react'

export const NAV_ITEMS: { label: string; path: string; icon: LucideIcon }[] = [
  { label: 'Dashboard', path: '/', icon: LayoutDashboard },
  { label: 'Marketplace', path: '/marketplace', icon: ShoppingBag },
  { label: 'Delegations', path: '/delegations', icon: Users },
  { label: 'Subscriptions', path: '/subscriptions', icon: Calendar },
  { label: 'Plans', path: '/plans', icon: ClipboardPen },
  { label: 'Faucet', path: '/faucet', icon: Droplets },
]
