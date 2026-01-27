# Multi-Delegator Client

A TypeScript/JavaScript client for interacting with the Multi-Delegator Solana program.

## Installation

```bash
npm install @multidelegator/client
```

## Usage

```typescript
import { MultiDelegatorClient } from "@multidelegator/client";
import { createRpc } from "gill";

// Initialize the client
const rpc = createRpc("https://api.mainnet-beta.solana.com");
const client = new MultiDelegatorClient(rpc);

// Initialize a multi-delegate account for a token
const signature = await client.initMultiDelegate(
  keypair,
  tokenMintAddress
);

// Create a simple delegation
const delegateSignature = await client.createSimpleDelegation(
  keypair,
  tokenMintAddress,
  delegateAddress,
  0, // Fixed delegation kind
  1_000_000, // amount
  3600 // expiry in seconds
);
```

## API Reference

### `MultiDelegatorClient`

#### `constructor(rpc: Rpc)`
Creates a new Multi-Delegator client instance.

#### `initMultiDelegate(owner, tokenMint)`
Initializes a multi-delegate account for the specified token mint and owner.

- `owner`: TransactionSigner - The owner's keypair/signer
- `tokenMint`: Address - The token mint address

#### `createSimpleDelegation(owner, tokenMint, delegate, kind, amount, expiryS)`
Creates a simple delegation to a delegate with a fixed amount and expiry.

- `owner`: TransactionSigner - The owner's keypair/signer
- `tokenMint`: Address - The token mint address
- `delegate`: Address - The delegate's wallet address
- `kind`: number - The delegation kind (0 = Fixed, 1 = Recurring)
- `amount`: number | bigint - Maximum amount to delegate
- `expiryS`: number | bigint - Expiry time in seconds

### PDA Functions

#### `getMultiDelegatePDA(user, tokenMint)`
Derives the MultiDelegate PDA for a user and token mint.

#### `getFixedDelegatePDA(multiDelegate, delegate, payer, kind)`
Derives the FixedDelegate PDA for a delegation.

### Utilities

#### `buildAndSendTransaction(rpc, instructions, signers)`
Builds and sends a transaction with the given instructions and signers.

## Generated Code

This package includes auto-generated TypeScript bindings for the Multi-Delegator program from Codama. These are exported from the main index and include:

- Instruction builders (`getInitMultiDelegateInstruction`, `getCreateSimpleDelegationInstruction`)
- Account types (`MultiDelegate`, `FixedDelegate`)
- Account fetchers (`fetchMultiDelegate`, etc.)

## License

MIT