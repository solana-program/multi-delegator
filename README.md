# Multi Delegator

Solana program for third party assets delegations.

## Overview

This repository hosts a Solana program built using Rust and the [Pinocchio](https://github.com/febo/pinocchio) library. It does **not** use the Anchor framework. It also includes a TypeScript client generated using [Codama](https://github.com/codama-js/codama).

## Project Structure

```
multidelegator/
├── client/                     # TypeScript client
│   ├── src/                    # Client source code
│   │   └── generated/          # Generated client code (from Codama)
│   ├── test/                   # Client integration tests
│   └── package.json            # Client dependencies and scripts
├── programs/
│   └── multi_delegator/        # Solana program (Rust)
│       ├── idl/                # Generated IDL (using Shank)
│       ├── src/                # Program source code
│       │   ├── instructions/   # Instruction handlers
│       │   ├── state/          # Program state structures
│       │   ├── tests/          # Program unit/integration tests (using LiteSVM)
│       │   └── lib.rs          # Program entry point
│       └── Cargo.toml          # Program configuration
├── Makefile                    # Build and test orchestration
├── README.md                   # This file
└── txtx.yml                    # txtx configuration used for deployments by surfpool
```

### Folder Descriptions

- **programs/multi_delegator/src/**: Contains all the source code for the Solana smart contract
  - **instructions/**: Handles different program instructions (actions the program can perform)
  - **instructions/helpers/**: Helper modules for program, system, and token operations
  - **state/**: Data structures representing program state (delegations, multi-delegates)
  - **tests/**: Comprehensive test suite including unit tests and integration tests
- **client/**: TypeScript client for interacting with the program
- **docs/**: Additional documentation including architecture details

## Getting Started

### Prerequisites

Ensure you have the following installed:

1. **Rust & Solana CLI**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   agave-install update
   ```

2. **Bun**:
   ```bash
   curl -fsSL https://bun.sh/install | bash
   ```

3. **Shank CLI** (for IDL generation):
   ```bash
   cargo install shank-cli
   ```

4. **Surfpool CLI** (for local validator and deployments):
   ```bash
   curl -sL https://run.surfpool.run/ | bash
   ```

5. **Codama CLI** (for TypeScript client generation):
   ```bash
   bun add -g @codama/cli
   ```

### Setup

Clone the repository and run the setup command:

```bash
git clone git@github.com:Moonsong-Labs/multi-delegator.git
cd multi-delegator
make setup
```

## Build & Test Commands

The project uses a `Makefile` to simplify common tasks.

- **Build All:** `make build` (builds program and client)
- **Build Program:** `make build-program`
- **Build Client:** `make build-client`
- **Test All:** `make test` (runs both program and client tests)
- **Test Program:** `make test-program`
- **Test Client:** `make test-client` (starts a local validator and runs client tests)
- **Generate IDL:** `make generate-idl`
- **Generate Client:** `make generate-client`
- **Clean:** `make clean`

### What happens when building and testing

- **Building:**
  1. The Solana program is compiled into a `.so` file.
  2. `shank` generates an IDL from the Rust source code.
  3. `codama` uses the IDL to generate a TypeScript client.
  4. The TypeScript client is built into the `dist/` directory.

- **Testing:**
  1. `make test-program` runs standard Rust SBF tests using `cargo test-sbf`.
  2. `make test-client` ensures a local validator (`surfpool`) is running, builds all dependencies, and runs the TypeScript integration tests using `bun test`.

## Codebase Overview

- **Framework:** Pinocchio (lightweight, zero-copy, compute-efficient).
- **Entrypoint:** `programs/multi_delegator/src/lib.rs` uses `entrypoint!(process_instruction)` and routes instructions based on a 1-byte discriminator.
- **Instructions:** Located in `programs/multi_delegator/src/instructions/`. Each instruction typically has:
  - A `process` function.
  - A `TryFrom<&[AccountInfo]>` implementation for account validation.
  - A `TryFrom<&[u8]>` implementation for data deserialization.
- **Testing:** Uses `litesvm` for fast, lightweight integration tests in Rust, and `bun test` for client-side integration tests.

## Code Style & Conventions

### Imports

- Group imports by crate.
- Prefer `pinocchio` types (`AccountInfo`, `Pubkey`, `ProgramResult`, `ProgramError`) over `solana_program` types when possible to maintain the lightweight nature.

### Formatting

- Follow standard Rust formatting (`cargo fmt`).
- Use 4 spaces for indentation.

### Naming

- **Structs/Traits:** PascalCase (e.g., `MakeAccounts`, `MakeInstructionData`).
- **Functions/Modules:** snake_case (e.g., `process_instruction`, `instructions::make`).
- **Constants:** SCREAMING_SNAKE_CASE.

### Account Validation

- Implement `TryFrom<&'a [AccountInfo]>` for a struct representing the instruction's accounts (e.g., `MakeAccounts`).
- Perform checks inside `try_from`:
  - `SignerAccount::check(account)?` for signers.
  - `MintInterface::check(account)?` for mints.
  - `AssociatedTokenAccount::check(...)` for ATAs.
  - Return `ProgramError::NotEnoughAccountKeys` if strict destructuring fails.

### Instruction Data

- Implement `TryFrom<&'a [u8]>` for a struct representing the instruction data.
- Validate data length and constraints immediately.
- Use `u64::from_le_bytes` for numeric deserialization.

### Error Handling

- Return `ProgramResult`.
- Use standard `ProgramError` variants where applicable.
- Define custom errors in `programs/multi_delegator/src/errors.rs` if necessary.

### Testing (`litesvm`)

- Rust tests are located in `programs/multi_delegator/src/tests/`.
- Use `litesvm` to simulate the chain.
- Helpers in `programs/multi_delegator/src/tests/utils.rs` (e.g., `setup`, `init_wallet`, `init_mint`, `build_and_send_transaction`) should be used to reduce boilerplate.

## Example Pattern

**Instruction Parsing:**

```rust
pub struct MyInstructionAccounts<'a> {
    pub signer: &'a AccountInfo,
    pub token_account: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for MyInstructionAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [signer, token_account] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        SignerAccount::check(signer)?;
        // ... other checks
        Ok(Self { signer, token_account })
    }
}
```

**Instruction Logic:**

```rust
pub fn process((data, accounts): (&[u8], &[AccountInfo])) -> ProgramResult {
    let accounts = MyInstructionAccounts::try_from(accounts)?;
    let data = MyInstructionData::try_from(data)?;
    // ... logic
    Ok(())
}
```
