# Multi-Delegator Client

TypeScript/JavaScript SDK for interacting with the Multi-Delegator Solana program.

This package exports:

- A high-level `MultiDelegatorClient` class in `src/client.ts`
- PDA helpers in `src/pdas.ts`
- Constants in `src/constants.ts`
- Codama-generated instruction/account bindings re-exported from `src/generated`

## Installation

```bash
npm install @multidelegator/client
```

## Quick Start

```typescript
import { createSolanaClient } from "gill";
import { MultiDelegatorClient } from "@multidelegator/client";

const solanaClient = createSolanaClient({ urlOrMoniker: "localnet" });
const client = new MultiDelegatorClient(solanaClient);
```

## Usage

```typescript
import {
  MultiDelegatorClient,
  getDelegationPDA,
  getMultiDelegatePDA,
} from "@multidelegator/client";
import { createSolanaClient } from "gill";

const solanaClient = createSolanaClient({ urlOrMoniker: "localnet" });
const client = new MultiDelegatorClient(solanaClient);

// Provided by your app/wallet flow:
// - owner: TransactionSigner
// - tokenMint: Address
// - userAta: Address
// - delegatee: Address

// 1) Initialize the MultiDelegate account for (owner, tokenMint)
const initResult = await client.initMultiDelegate(owner, tokenMint, userAta);
console.log(initResult.signature);

// 2) Create a fixed delegation
const fixedResult = await client.createFixedDelegation(
  owner,
  tokenMint,
  delegatee,
  0n, // nonce
  1_000_000n, // amount
  BigInt(Math.floor(Date.now() / 1000) + 3600), // expiryTs
);
console.log(fixedResult.signature);

// 3) Derive delegation PDA (used by transfer/revoke flows)
const [multiDelegate] = await getMultiDelegatePDA(owner.address, tokenMint);
const [delegationPda] = await getDelegationPDA(
  multiDelegate,
  owner.address,
  delegatee,
  0n,
);
console.log(delegationPda);
```

## API Reference

### `MultiDelegatorClient`

#### `constructor(client)`
Creates a client using an object compatible with Gill's `createSolanaClient(...)` result:

- `client.rpc`
- `client.sendAndConfirmTransaction(...)`

#### `initMultiDelegate(owner, tokenMint, userAta, tokenProgram?)`
Initializes the per-`(user, mint)` MultiDelegate PDA and configures token delegation.

- `owner`: `TransactionSigner`
- `tokenMint`: `Address`
- `userAta`: `Address`
- `tokenProgram?`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `createFixedDelegation(delegator, tokenMint, delegatee, nonce, amount, expiryTs)`
Creates a fixed delegation.

- `delegator`: `TransactionSigner`
- `tokenMint`: `Address`
- `delegatee`: `Address`
- `nonce`: `number | bigint`
- `amount`: `number | bigint`
- `expiryTs`: `number | bigint`
- Returns: `Promise<{ signature: string }>`

#### `createRecurringDelegation(delegator, tokenMint, delegatee, nonce, amountPerPeriod, periodLengthS, startTs, expiryTs)`
Creates a recurring delegation.

- `delegator`: `TransactionSigner`
- `tokenMint`: `Address`
- `delegatee`: `Address`
- `nonce`: `number | bigint`
- `amountPerPeriod`: `number | bigint`
- `periodLengthS`: `number | bigint`
- `startTs`: `number | bigint`
- `expiryTs`: `number | bigint`
- Returns: `Promise<{ signature: string }>`

#### `revokeDelegation(delegator, delegationAccount)`
Closes an existing delegation account.

- `delegator`: `TransactionSigner`
- `delegationAccount`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `transferFixed(delegatee, delegator, delegatorAta, tokenMint, delegationPda, amount, receiverAta)`
Transfers tokens through a fixed delegation.

- `delegatee`: `TransactionSigner`
- `delegator`: `Address`
- `delegatorAta`: `Address`
- `tokenMint`: `Address`
- `delegationPda`: `Address`
- `amount`: `number | bigint`
- `receiverAta`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `transferRecurring(delegatee, delegator, delegatorAta, tokenMint, delegationPda, amount, receiverAta)`
Transfers tokens through a recurring delegation.

- Same parameters as `transferFixed(...)`
- Returns: `Promise<{ signature: string }>`

#### `getDelegationsForWallet(wallet)`
Returns decoded fixed/recurring delegation accounts for a delegator wallet.

- `wallet`: `Address`
- Returns: `Promise<Delegation[]>`

#### `isMultiDelegateInitialized(user, tokenMint)`
Checks whether the MultiDelegate PDA exists for `(user, tokenMint)`.

- `user`: `Address`
- `tokenMint`: `Address`
- Returns: `Promise<{ initialized: boolean; pda: Address }>`

### Exported Types

- `Delegation`:
  - `{ kind: "fixed"; address: Address; data: FixedDelegation }`
  - `{ kind: "recurring"; address: Address; data: RecurringDelegation }`

### PDA Helpers

#### `getMultiDelegatePDA(user, tokenMint)`
Derives the MultiDelegate PDA.

#### `getDelegationPDA(multiDelegate, delegator, delegatee, nonce)`
Derives the delegation PDA for fixed or recurring delegations.

### Constants

From `src/constants.ts`:

- `PROGRAM_ID`
- `KIND_DISCRIMINATOR_OFFSET`
- `DELEGATOR_OFFSET`
- `DELEGATEE_OFFSET`
- `U64_BYTE_SIZE`
- `MULTI_DELEGATE_SEED`
- `DELEGATION_SEED`
- `DELEGATION_KINDS`
- `DelegationKindId`

## Generated Bindings (Codama)

The package re-exports generated program bindings from `src/generated`, including instruction builders and account helpers used by the high-level client, such as:

- Instruction builders:
  - `getInitMultiDelegateInstruction`
  - `getCreateFixedDelegationInstruction`
  - `getCreateRecurringDelegationInstruction`
  - `getRevokeDelegationInstruction`
  - `getTransferFixedInstruction`
  - `getTransferRecurringInstruction`
- Account helpers:
  - `fetchMultiDelegate`
  - `fetchFixedDelegation`
  - `fetchRecurringDelegation`

## Contributor Note

Generated bindings are produced by Codama and are gitignored in this repository (`clients/typescript/src/generated`).

To regenerate from the repo root:

```bash
make generate-client
```

Or directly:

```bash
bun run generate
```

## License

MIT