import { Toaster } from './ui/sonner'
import { AppHeader } from './app-header'
import { AppFooter } from './app-footer'
import React from 'react'
import { ClusterChecker } from './cluster/cluster-ui'
import { AccountChecker } from './account/account-ui'

export function AppLayout({
  children,
  links,
}: {
  children: React.ReactNode
  links: { label: string; path: string }[]
}) {
  return (
    <div className="flex flex-col min-h-screen">
      <AppHeader links={links} />
      <main className="flex-grow container mx-auto p-4">
        <ClusterChecker>
          <AccountChecker />
        </ClusterChecker>
        {children}
      </main>
      <AppFooter />
      <Toaster />
    </div>
  )
}
