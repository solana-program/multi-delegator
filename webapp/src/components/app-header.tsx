import { useState } from 'react'
import { Button } from './ui/button'
import { Menu, X } from 'lucide-react'
import { ClusterButton, WalletButton } from './solana/solana-provider'
import { TimeTravelButton } from './time-travel/time-travel-button'
import { Link, useLocation } from 'react-router'
import { NAV_ITEMS } from './nav-items'

export function AppHeader() {
  const { pathname } = useLocation()
  const [showMenu, setShowMenu] = useState(false)

  function isActive(path: string) {
    return path === '/' ? pathname === '/' : pathname.startsWith(path)
  }

  return (
    <header className="relative z-50 px-4 py-2 bg-neutral-100 dark:bg-neutral-900 dark:text-neutral-400">
      <div className="mx-auto flex justify-between items-center">
        <span className="text-xl md:hidden">MultiDelegator</span>

        <Button variant="ghost" size="icon" className="md:hidden" onClick={() => setShowMenu(!showMenu)}>
          {showMenu ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
        </Button>

        <div className="hidden md:flex items-center gap-4 ml-auto">
          <TimeTravelButton />
          <WalletButton />
          <ClusterButton />
        </div>

        {showMenu && (
          <div className="md:hidden fixed inset-x-0 top-[52px] bottom-0 bg-neutral-100/95 dark:bg-neutral-900/95 backdrop-blur-sm">
            <div className="flex flex-col p-4 gap-4 border-t dark:border-neutral-800">
              <ul className="flex flex-col gap-4">
                {NAV_ITEMS.map(({ label, path, icon: Icon }) => (
                  <li key={path}>
                    <Link
                      className={`flex items-center gap-3 hover:text-neutral-500 dark:hover:text-white text-lg py-2 ${isActive(path) ? 'text-neutral-500 dark:text-white' : ''}`}
                      to={path}
                      onClick={() => setShowMenu(false)}
                    >
                      <Icon className="h-5 w-5" />
                      {label}
                    </Link>
                  </li>
                ))}
              </ul>
              <div className="flex flex-col gap-4">
                <TimeTravelButton />
                <WalletButton />
                <ClusterButton />
              </div>
            </div>
          </div>
        )}
      </div>
    </header>
  )
}
