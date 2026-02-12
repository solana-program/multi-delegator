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
├── Makefile                       # Build/test/dev task entrypoint
└── codama.js                      # Codama generation config
```

## Quick Start

```bash
git clone git@github.com:Moonsong-Labs/multi-delegator.git
cd multi-delegator
make setup

# If missing, create the local development keypair expected by the Makefile
mkdir -p keys
[ -f keys/multi_delegator-keypair.json ] || solana-keygen new --no-bip39-passphrase -o keys/multi_delegator-keypair.json

make build
make test-program
```

For the full suite (program + client tests):

```bash
make test
```

## Prerequisites

`make setup` checks for these tools: `bun`, `cargo`, `shank`, `solana-keygen`, and `surfpool`.

Install the toolchain:

1. Rust
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Solana CLI (includes `solana-keygen` and `solana-test-validator`)
   ```bash
   sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
   ```
3. Bun
   ```bash
   curl -fsSL https://bun.sh/install | bash
   ```
4. Shank CLI
   ```bash
   cargo install shank-cli
   ```
5. Surfpool CLI
   ```bash
   curl -sL https://run.surfpool.run/ | bash
   ```
6. Node.js + npm (required by `webapp/` scripts)

## Keypair and Program ID

Local workflows use `keys/multi_delegator-keypair.json` as the source keypair for deployment artifacts and program ID derivation in the `Makefile`.

- If the file exists, `make build` and validator scripts reuse it.
- If the file is missing, generate it before build/test commands:

```bash
mkdir -p keys
solana-keygen new --no-bip39-passphrase -o keys/multi_delegator-keypair.json
```

The keypair is automatically copied into the deploy directory when you run `make build` or `make build-program`.

## Build and Test

The `Makefile` is the main entrypoint for day-to-day development.

- `make build` builds program, clients, and webapp
- `make build-program` compiles the SBF program
- `make generate-idl` regenerates `programs/multi_delegator/idl/multi_delegator.json`
- `make generate-client` regenerates clients from IDL via Codama
- `make build-client` builds `clients/typescript` into `clients/typescript/dist`
- `make test-program` runs Rust SBF tests (`cargo test-sbf`)
- `make test-client` runs TypeScript integration tests (`bun test`)
- `make test` runs setup + program + client tests
- `make fmt-check` and `make lint-check` run formatting/lint checks

### Validator Modes

Two local validator flows are used in this repo:

- `make test-client` uses `surfpool` (auto-start via `ensure-surfpool` in `Makefile`)
- `make webapp` uses `solana-test-validator` (via `scripts/start-webapp.sh`)

Both default to `http://localhost:8899`, but they are started by different tooling.

## Webapp Demo (Quickstart)

The demo app in `webapp/` provides a local UI + local API for development flows (including faucet utilities).

```bash
make build          # builds program, clients, and webapp (includes npm install)
make webapp         # starts validator + init + API + web UI
```

Expected local endpoints:

- Validator RPC: `http://localhost:8899`
- API: `http://localhost:3001`
- Web UI: `http://localhost:5173`

Stop local validators:

```bash
make kill-validator
```
