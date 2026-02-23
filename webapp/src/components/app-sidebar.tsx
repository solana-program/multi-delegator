import { Link, useLocation } from 'react-router'
import { NAV_ITEMS } from './nav-items'
import solanaLogo from '@/assets/solana-logo.svg'

export function AppSidebar() {
  const { pathname } = useLocation()

  function isActive(path: string) {
    return path === '/' ? pathname === '/' : pathname.startsWith(path)
  }

  return (
    <aside className="hidden md:flex flex-col w-64 shrink-0 bg-[#0a0a0a] text-sidebar-foreground border-r border-white/5">
      <div className="flex items-center gap-2 px-6 pt-8 pb-4">
        <span className="text-xl font-bold text-white tracking-tight">Multi Delegator</span>
        <img src={solanaLogo} alt="Solana" className="h-5 w-5 shrink-0" />
      </div>
      <nav className="flex flex-col gap-2 px-4 pt-8">
        {NAV_ITEMS.map(({ label, path, icon: Icon }) => {
          const active = isActive(path)
          return (
            <Link
              key={path}
              to={path}
              className={`flex items-center gap-4 px-4 py-3.5 rounded-full text-[15px] font-medium transition-all duration-300 ${
                active
                  ? 'bg-gradient-to-r from-purple-500/20 to-transparent border border-purple-500/30 text-white shadow-[0_0_20px_rgba(168,85,247,0.15)] backdrop-blur-md relative'
                  : 'text-gray-400 hover:text-gray-200 hover:bg-white/5'
              }`}
            >
              {active && (
                <div className="absolute inset-0 rounded-full shadow-[0_0_15px_rgba(168,85,247,0.3)] opacity-50 pointer-events-none" />
              )}
              <Icon className={`h-5 w-5 shrink-0 z-10 ${active ? 'text-white' : 'text-gray-500'}`} strokeWidth={active ? 2 : 1.5} />
              <span className="z-10">{label}</span>
            </Link>
          )
        })}
      </nav>
    </aside>
  )
}
