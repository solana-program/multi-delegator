import { Route, Routes } from 'react-router'
import { AppProviders } from '@/components/app-providers'
import { AppLayout } from '@/components/app-layout'
import { Dashboard } from '@/routes/dashboard'
import { Marketplace } from '@/routes/marketplace'
import { Faucet } from '@/routes/faucet'
import { Delegations } from '@/routes/delegations'
import { Subscriptions } from '@/routes/subscriptions'
import { Plans } from '@/routes/plans'

export default function App() {
  return (
    <AppProviders>
      <AppLayout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/marketplace" element={<Marketplace />} />
          <Route path="/delegations" element={<Delegations />} />
          <Route path="/subscriptions" element={<Subscriptions />} />
          <Route path="/plans" element={<Plans />} />
          <Route path="/faucet" element={<Faucet />} />
          <Route path="*" element={<Dashboard />} />
        </Routes>
      </AppLayout>
    </AppProviders>
  )
}
