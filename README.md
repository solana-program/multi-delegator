# Multi Delegator Webapp

> **⚠️ Note:** This webapp is currently in **Beta mode**. Features and capabilities are subject to change.

A React/Vite demonstration web application for the Solana Multi-Delegator program. This app provides a comprehensive UI to interact with the multi-delegator smart contract, allowing users to manage recurring subscriptions, delegate tokens, and test the full lifecycle of managed delegations on Solana.

## Features & Capabilities

The webapp is built with React, Vite, `@solana/kit`, `@tanstack/react-query`, Jotai, and TailwindCSS/shadcn. It includes the following core capabilities:

### 1. Program Management & Setup
- **Program Deployment:** Deploy the multi-delegator Solana program directly from the UI.
- **Program Status:** Check and monitor the on-chain status of the deployed program.
- **Setup Wizard:** An onboarding flow to initialize the environment, configure tokens, and set up necessary on-chain state.

### 2. Subscription Plans & Marketplace
- **Plan Management:** Merchants and creators can create, configure, and manage recurring subscription plans.
- **Plan Marketplace:** A storefront directory where users can browse and subscribe to available plans.

### 3. Subscriptions & Delegations
- **My Subscriptions:** View and manage active plan subscriptions.
- **Delegation Management:** Delegate funds or authority to the smart contract to handle recurring payments automatically.
- **Active Delegations Dashboard:** Monitor active delegations, their current balances, and status.

### 4. Payment Collection
- **Collect Payments:** A dedicated interface for merchants to trigger the collection of due payments from their subscribers based on active delegations.

### 5. Developer & Testing Tools
- **Time Travel:** A specialized testing utility to simulate the passage of time on localnet/devnet, allowing developers to test recurring billing cycles without waiting.
- **Faucet:** Built-in tool to request test SOL or SPL tokens for testing purposes.
- **Cluster Switching:** Easily switch between different Solana networks (e.g., Localnet, Devnet, Mainnet).

### 6. Account & Wallet Integration
- **Wallet Connection:** Standard Solana wallet connection and transaction signing capabilities.
- **Account Dashboard:** High-level overview of the user's account, showing balances, active plans, and recent activity.

## Quick Start

### Prerequisites
- Node.js (v18+)
- pnpm or npm

### Installation

```bash
# Install dependencies
pnpm install

# Start the development server
pnpm run dev
```

### Local API & Validator
The webapp includes a local API (`api/`) and scripts to run a local Solana test validator for development.

```bash
# Start the local validator and API
pnpm run start:api
```

Expected local endpoints:
- Web UI: `http://localhost:5173`
- API: `http://localhost:3001`
- Validator RPC: `http://localhost:8899`
