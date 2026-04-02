import { MULTI_DELEGATOR_PROGRAM_ADDRESS } from './generated/index.js';

// Program ID declared via `declare_id!` macro
// See: programs/multi_delegator/src/lib.rs
export const PROGRAM_ID = MULTI_DELEGATOR_PROGRAM_ADDRESS;

// See: programs/multi_delegator/src/state/header.rs
export const CURRENT_PROGRAM_VERSION = 1;

// Default zero address used for padding arrays
export const ZERO_ADDRESS =
  '11111111111111111111111111111111' as import('gill').Address<'11111111111111111111111111111111'>;

// Header struct layout offsets
// Layout: discriminator (1) + version (1) + bump (1) + delegator (32) + delegatee (32) + payer (32) + init_id (8) = 107 bytes
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
export const EVENT_AUTHORITY_SEED = 'event_authority';

// Plan: discriminator(1) + owner(32) + bump(1) + status(1) + planData(448)
export const PLAN_SIZE = 483;
// Subscription: header(107) + amountPulledInPeriod(8) + currentPeriodStartTs(8) + expiresAtTs(8)
export const SUBSCRIPTION_SIZE = 131;

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
