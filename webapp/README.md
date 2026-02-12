# Multi Delegator Webapp

Web interface for managing Solana token delegations (USDC). Connects to the Multi Delegator on-chain program, allowing users to create, manage, and revoke delegations with controlled spending limits and time-based expiry.

## Features

- **Wallet Connection** - Solana wallet integration (tested with Phantom) with real-time SOL and USDC balance display
- **Create Delegations** - Two delegation types:
  - **Fixed**: one-time total amount with an expiry date
  - **Recurring**: per-period amount with configurable period length
- **View Delegations** - Separate tabs for outgoing (delegator) and incoming (delegatee) delegations, with active/expired filtering
- **Revoke Delegations** - Cancel active outgoing delegations on-chain
- **Transfer Under Delegation** - Delegatees can withdraw amounts within the delegation rules
- **MDA Initialization** - Multi Delegate Account setup flow required before creating delegations
- **Dev Faucet** - Request SOL/USDC airdrops for local testing (hidden on mainnet)
- **Theme Support** - Dark/light mode toggle

## Scripts

| Script | Description |
|--------|-------------|
| `npm run dev` | Start the Vite dev server with hot module replacement |
| `npm run build` | Type-check with TypeScript and build for production |
| `npm run lint` | Run ESLint across the project |
| `npm run preview` | Preview the production build locally |

From the project root, `just webapp` builds the program and clients, starts a local validator + API, and launches the webapp.

## Tech Stack

React 19, TypeScript, Vite, Tailwind CSS, Radix UI, jotai (state), TanStack Query (data fetching), Solana Kit, Wallet UI adapter.
