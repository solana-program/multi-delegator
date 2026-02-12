# Multi Delegator

Solana program and clients for managed delegations on both SPL Token and Token-2022.

## Overview

For each `(user, mint)` pair, the program creates a Multi Delegate PDA and sets it as the single delegate authority on the user's token account. This works for both token programs: SPL Token and Token-2022.

The program then routes transfer requests through delegation accounts and enforces delegation rules before any token movement.

Currently supported delegation kinds:

- **Fixed delegation**: authorize a delegatee to spend up to a total amount until an expiry time.
- **Recurring delegation**: authorize a delegatee to spend up to a per-period amount that resets each period until expiry.

This repository contains:

- A Rust Solana program built with [Pinocchio](https://github.com/febo/pinocchio)
- IDL generation via [Shank](https://github.com/metaplex-foundation/shank)
- Generated clients via [Codama](https://github.com/codama-js/codama):
  - TypeScript client in `clients/typescript`
  - Rust client in `clients/rust`
- A local demo webapp in `webapp/`

## Project Structure

```text
multi-delegator/
├── programs/multi_delegator/     # Rust Solana program
│   ├── src/
│   │   ├── instructions/          # Instruction handlers + helpers
│   │   ├── state/                 # Account/state types (fixed, recurring, header, MDA)
│   │   └── tests/                 # Rust tests (litesvm)
│   └── idl/                       # Shank-generated IDL
├── clients/
│   ├── typescript/                # TypeScript SDK + tests
│   └── rust/                      # Rust generated client
├── webapp/                        # Local demo UI + local API
├── scripts/                       # Validator / full-stack launcher scripts
├── docs/                          # Architecture docs
├── runbooks/                      # Deployment runbooks
├── justfile                       # Build/test/dev task entrypoint
└── codama.js                      # Codama generation config
```

## Quick Start

```bash
git clone git@github.com:Moonsong-Labs/multi-delegator.git
cd multi-delegator
just setup

# If missing, create the local development keypair expected by the justfile
mkdir -p keys
[ -f keys/multi_delegator-keypair.json ] || solana-keygen new --no-bip39-passphrase -o keys/multi_delegator-keypair.json

just build
just test-program
```

For the full suite (program + client tests):

```bash
just test
```

## Prerequisites

`just setup` checks for these tools: `bun`, `cargo`, `pnpm`, `solana-keygen`, and `surfpool`.

Install the toolchain:

1. Rust
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Solana CLI (includes `solana-keygen` and `solana-test-validator`)
   ```bash
   sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
   ```
3. Pnpm
   ```bash
   curl -fsSL https://get.pnpm.io/install.sh | sh -
   ```
4. Just
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/.local/bin
   ```
5. Surfpool CLI
   ```bash
   curl -sL https://run.surfpool.run/ | bash
   ```
6. Node.js + npm (required by `webapp/` scripts)

## Keypair and Program ID

Local workflows use `keys/multi_delegator-keypair.json` as the source keypair for deployment artifacts and program ID derivation in the `justfile`.

- If the file exists, `just build` and validator scripts reuse it.
- If the file is missing, generate it before build/test commands:

```bash
mkdir -p keys
solana-keygen new --no-bip39-passphrase -o keys/multi_delegator-keypair.json
```

The keypair is automatically copied into the deploy directory when you run `just build` or `just build-program`.

## Build and Test

The `justfile` is the main entrypoint for day-to-day development.

- `just build` builds program, clients, and webapp
- `just build-program` compiles the SBF program
- `just generate-idl` regenerates `programs/multi_delegator/idl/multi_delegator.json`
- `just generate-client` regenerates clients from IDL via Codama
- `just build-client` builds `clients/typescript` into `clients/typescript/dist`
- `just test-program` runs Rust SBF tests (`cargo test-sbf`)
- `just test-client` runs TypeScript integration tests (`bun test`)
- `just test` runs setup + program + client tests
- `just fmt-check` and `just lint-check` run formatting/lint checks

### Validator Modes

Two local validator flows are used in this repo:

- `just test-client` uses `surfpool` (auto-start via `ensure-surfpool` in `justfile`)
- `just webapp` uses `solana-test-validator` (via `scripts/start-webapp.sh`)

Both default to `http://localhost:8899`, but they are started by different tooling.

## Webapp Demo (Quickstart)

The demo app in `webapp/` provides a local UI + local API for development flows (including faucet utilities).

```bash
just build          # builds program, clients, and webapp (includes npm install)
just webapp         # starts validator + init + API + web UI
```

Expected local endpoints:

- Validator RPC: `http://localhost:8899`
- API: `http://localhost:3001`
- Web UI: `http://localhost:5173`

Stop local validators:

```bash
just kill-validator
```
