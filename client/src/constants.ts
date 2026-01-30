import { MULTI_DELEGATOR_PROGRAM_ADDRESS } from './generated/index.js';

// Program ID declared via `declare_id!` macro
// See: programs/multi_delegator/src/lib.rs
export const PROGRAM_ID = MULTI_DELEGATOR_PROGRAM_ADDRESS;

// Byte offset of the `delegator` field in the Header struct
// Header layout: version (1 byte) + kind (1 byte) + bump (1 byte) + delegator (32 bytes)
// See: programs/multi_delegator/src/state/header.rs
export const DELEGATOR_OFFSET = 3;

// PDA seed for MultiDelegate account
// See: programs/multi_delegator/src/state/multi_delegate.rs (MultiDelegate::SEED)
export const MULTI_DELEGATE_SEED = 'MultiDelegate';

// PDA seed for delegation accounts (FixedDelegation, RecurringDelegation)
// See: programs/multi_delegator/src/state/common.rs (DELEGATE_BASE_SEED)
export const DELEGATION_SEED = 'delegation';
