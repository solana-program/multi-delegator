import { MULTI_DELEGATOR_PROGRAM_ADDRESS } from './generated/index.js';

// Program ID declared via `declare_id!` macro
// See: programs/multi_delegator/src/lib.rs
export const PROGRAM_ID = MULTI_DELEGATOR_PROGRAM_ADDRESS;

// Header struct layout offsets
// Layout: discriminator (1 byte) + version (1 byte) + bump (1 byte) + delegator (32 bytes) + delegatee (32 bytes)
// See: programs/multi_delegator/src/state/header.rs
export const DISCRIMINATOR_OFFSET = 0;
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

export const PLAN_SEED = 'plan';
export const SUBSCRIPTION_SEED = 'subscription';

export const PLAN_SIZE = 483;
export const SUBSCRIPTION_SIZE = 123;

export const PLAN_OWNER_OFFSET = 1;

export const MAX_PLAN_DESTINATIONS = 4;
export const MAX_PLAN_PULLERS = 4;
export const METADATA_URI_LEN = 128;

// Delegation kinds with metadata for UI display
// Maps to AccountDiscriminator enum in programs/multi_delegator/src/state/common.rs
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
  subscription: {
    id: 'subscription',
    label: 'Subscription',
    description: 'Plan-based recurring subscription delegation',
    icon: 'CalendarCheck',
  },
} as const;

export type DelegationKindId = keyof typeof DELEGATION_KINDS;
