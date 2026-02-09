import { MULTI_DELEGATOR_PROGRAM_ADDRESS } from './generated/index.js';

// Program ID declared via `declare_id!` macro
// See: programs/multi_delegator/src/lib.rs
export const PROGRAM_ID = MULTI_DELEGATOR_PROGRAM_ADDRESS;

// Header struct layout offsets
// Layout: version (1 byte) + kind (1 byte) + bump (1 byte) + delegator (32 bytes) + delegatee (32 bytes)
// See: programs/multi_delegator/src/state/header.rs
export const KIND_DISCRIMINATOR_OFFSET = 1;
export const DELEGATOR_OFFSET = 3;
export const DELEGATEE_OFFSET = 35;

// Byte size for u64 values (used in nonce encoding)
export const U64_BYTE_SIZE = 8;

// PDA seed for MultiDelegate account
// See: programs/multi_delegator/src/state/multi_delegate.rs (MultiDelegate::SEED)
export const MULTI_DELEGATE_SEED = 'MultiDelegate';

// PDA seed for delegation accounts (FixedDelegation, RecurringDelegation)
// See: programs/multi_delegator/src/state/common.rs (DELEGATE_BASE_SEED)
export const DELEGATION_SEED = 'delegation';

// Delegation kinds with metadata for UI display
// Maps to DelegationKind enum in programs/multi_delegator/src/state/common.rs
export const DELEGATION_KINDS = {
  fixed: {
    id: 'fixed',
    label: 'Fixed',
    description: 'One-time delegation with a fixed total amount',
    icon: 'Coins', // lucide-react icon name
  },
  recurring: {
    id: 'recurring',
    label: 'Recurring',
    description: 'Periodic delegation with amount per time period',
    icon: 'RefreshCw', // lucide-react icon name
  },
} as const;

export type DelegationKindId = keyof typeof DELEGATION_KINDS;
