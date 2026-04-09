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
const initResult = await client.initMultiDelegate({
  owner,
  tokenMint,
  userAta,
  tokenProgram,
});
console.log(initResult.signature);

// 2) Create a fixed delegation
const fixedResult = await client.createFixedDelegation({
  delegator: owner,
  tokenMint,
  delegatee,
  nonce: 0n,
  amount: 1_000_000n,
  expiryTs: BigInt(Math.floor(Date.now() / 1000) + 3600),
});
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

#### `initMultiDelegate(params)`
Initializes the per-`(user, mint)` MultiDelegate PDA and configures token delegation.

- `params.owner`: `TransactionSigner`
- `params.tokenMint`: `Address`
- `params.userAta`: `Address`
- `params.tokenProgram`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `closeMultiDelegate(params)`
Closes the MultiDelegate PDA, returning rent to the user. Invalidates all existing delegations on re-initialization (emergency kill switch).

- `params.user`: `TransactionSigner`
- `params.tokenMint`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `createFixedDelegation(params)`
Creates a fixed delegation.

- `params.delegator`: `TransactionSigner`
- `params.tokenMint`: `Address`
- `params.delegatee`: `Address`
- `params.nonce`: `number | bigint`
- `params.amount`: `number | bigint`
- `params.expiryTs`: `number | bigint`
- Returns: `Promise<{ signature: string }>`

#### `createRecurringDelegation(params)`
Creates a recurring delegation.

- `params.delegator`: `TransactionSigner`
- `params.tokenMint`: `Address`
- `params.delegatee`: `Address`
- `params.nonce`: `number | bigint`
- `params.amountPerPeriod`: `number | bigint`
- `params.periodLengthS`: `number | bigint`
- `params.startTs`: `number | bigint`
- `params.expiryTs`: `number | bigint`
- Returns: `Promise<{ signature: string }>`

#### `revokeDelegation(params)`
Closes an existing delegation account, returning rent to the original payer.

- `params.authority`: `TransactionSigner`
- `params.delegationAccount`: `Address`
- `params.receiver?`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `transferFixed(params)`
Transfers tokens through a fixed delegation.

- `params.delegatee`: `TransactionSigner`
- `params.delegator`: `Address`
- `params.delegatorAta`: `Address`
- `params.tokenMint`: `Address`
- `params.delegationPda`: `Address`
- `params.amount`: `number | bigint`
- `params.receiverAta`: `Address`
- `params.tokenProgram`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `transferRecurring(params)`
Transfers tokens through a recurring delegation.

- Same parameters as `transferFixed(params)`
- Returns: `Promise<{ signature: string }>`

#### `createPlan(params)`
Creates a subscription plan.

- `params.owner`: `TransactionSigner`
- `params.planId`: `number | bigint`
- `params.mint`: `Address`
- `params.amount`: `number | bigint`
- `params.periodHours`: `number | bigint`
- `params.endTs`: `number | bigint`
- `params.destinations`: `Address[]`
- `params.pullers`: `Address[]`
- `params.metadataUri`: `string`
- Returns: `Promise<{ signature: string; planPda: Address }>`

#### `updatePlan(params)`
Updates a plan's status, endTs, metadata, or pullers.

- `params.owner`: `TransactionSigner`
- `params.planPda`: `Address`
- `params.status`: `PlanStatus`
- `params.endTs`: `number | bigint`
- `params.metadataUri`: `string`
- `params.pullers?`: `Address[]`
- Returns: `Promise<{ signature: string }>`

#### `subscribe(params)`
Subscribes to a plan, creating a SubscriptionDelegation PDA.

- `params.subscriber`: `TransactionSigner`
- `params.merchant`: `Address`
- `params.planId`: `number | bigint`
- `params.tokenMint`: `Address`
- Returns: `Promise<{ signature: string; subscriptionPda: Address }>`

#### `cancelSubscription(params)`
Cancels a subscription.

- `params.subscriber`: `TransactionSigner`
- `params.planPda`: `Address`
- `params.subscriptionPda`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `transferSubscription(params)`
Transfers tokens from a subscription delegation.

- `params.caller`: `TransactionSigner`
- `params.delegator`: `Address`
- `params.tokenMint`: `Address`
- `params.subscriptionPda`: `Address`
- `params.planPda`: `Address`
- `params.amount`: `number | bigint`
- `params.receiverAta`: `Address`
- `params.tokenProgram`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `deletePlan(params)`
Deletes an expired plan, recovering rent.

- `params.owner`: `TransactionSigner`
- `params.planPda`: `Address`
- Returns: `Promise<{ signature: string }>`

#### `getDelegationsForWallet(wallet)`
Returns decoded fixed/recurring/subscription delegation accounts for a delegator wallet.

- `wallet`: `Address`
- Returns: `Promise<Delegation[]>`

#### `getDelegationsAsDelegatee(wallet)`
Returns decoded delegation accounts where the wallet is the delegatee.

- `wallet`: `Address`
- Returns: `Promise<Delegation[]>`

#### `getPlansForOwner(owner)`
Returns all plans owned by the given address.

- `owner`: `Address`
- Returns: `Promise<PlanWithAddress[]>`

#### `getActiveDelegationSummary(wallet)`
Returns a summary of active delegations and subscriptions for a wallet. Useful for checking outstanding commitments before closing a MultiDelegate.

- `wallet`: `Address`
- Returns: `Promise<{ fixed: number; recurring: number; subscriptions: number; total: number }>`

#### `isMultiDelegateInitialized(user, tokenMint)`
Checks whether the MultiDelegate PDA exists for `(user, tokenMint)`.

- `user`: `Address`
- `tokenMint`: `Address`
- Returns: `Promise<{ initialized: boolean; pda: Address }>`

### Exported Types

- `Delegation`:
  - `{ kind: "fixed"; address: Address; data: FixedDelegation }`
  - `{ kind: "recurring"; address: Address; data: RecurringDelegation }`
  - `{ kind: "subscription"; address: Address; data: SubscriptionDelegation }`
- `PlanWithAddress`: `{ address: Address; data: Plan }`
- `DelegationKindId`: `"fixed" | "recurring" | "subscription"`

### PDA Helpers

#### `getMultiDelegatePDA(user, tokenMint)`
Derives the MultiDelegate PDA.

#### `getDelegationPDA(multiDelegate, delegator, delegatee, nonce)`
Derives the delegation PDA for fixed or recurring delegations.

#### `getPlanPDA(owner, planId)`
Derives the Plan PDA.

#### `getSubscriptionPDA(planPda, subscriber)`
Derives the SubscriptionDelegation PDA.

#### `getEventAuthorityPDA(programId?)`
Derives the event authority PDA used for self-CPI event emission.

### Constants

From `src/constants.ts`:

- `PROGRAM_ID`
- `CURRENT_PROGRAM_VERSION`
- `ZERO_ADDRESS`
- `DISCRIMINATOR_OFFSET`
- `DELEGATOR_OFFSET`
- `DELEGATEE_OFFSET`
- `U64_BYTE_SIZE`
- `MULTI_DELEGATE_SEED`
- `DELEGATION_SEED`
- `PLAN_SEED`
- `SUBSCRIPTION_SEED`
- `EVENT_AUTHORITY_SEED`
- `PLAN_SIZE`
- `SUBSCRIPTION_SIZE`
- `PLAN_OWNER_OFFSET`
- `MAX_PLAN_DESTINATIONS`
- `MAX_PLAN_PULLERS`
- `METADATA_URI_LEN`
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
  - `getCreatePlanInstruction`
  - `getUpdatePlanInstruction`
  - `getSubscribeInstruction`
  - `getCancelSubscriptionInstruction`
  - `getTransferSubscriptionInstruction`
  - `getDeletePlanInstruction`
- Account helpers:
  - `fetchMultiDelegate`
  - `fetchFixedDelegation`
  - `fetchRecurringDelegation`
  - `fetchSubscriptionDelegation`
  - `fetchPlan`

## Contributor Note

Generated bindings are produced by Codama and are gitignored in this repository (`clients/typescript/src/generated`).

To regenerate from the repo root:

```bash
just generate-client
```

Or directly:

```bash
bun run generate
```

## License

MIT