import { Route, Routes } from 'react-router'
import { AppProviders } from '@/components/app-providers'
import { AppLayout } from '@/components/app-layout'
import { Dashboard } from '@/routes/dashboard'
import { Faucet } from '@/routes/faucet'

export default function App() {
  return (
    <AppProviders>
      <AppLayout links={[{ label: 'Faucet', path: '/faucet' }, { label: 'Dashboard', path: '/' }]}>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/faucet" element={<Faucet />} />
        </Routes>
      </AppLayout>
    </AppProviders>
  )
}
