pub mod close_multidelegate;
pub mod create_fixed_delegation;
pub use create_fixed_delegation::CreateFixedDelegationData;
pub mod create_recurring_delegation;
pub use create_recurring_delegation::CreateRecurringDelegationData;
pub mod helpers;
pub mod initialize_multidelegate;
pub mod revoke_delegation;
pub mod transfer_fixed_delegation;
pub mod transfer_recurring_delegation;

pub use helpers::*;

use core::fmt;

use codama::CodamaInstructions;
use pinocchio::error::ProgramError;

use crate::MultiDelegatorError;

#[derive(Debug, CodamaInstructions)]
#[repr(u8)]
pub enum MultiDelegatorInstruction {
    #[codama(account(
        name = "owner",
        signer,
        writable,
        docs = "The owner of the multi-delegate program"
    ))]
    #[codama(account(
        name = "multi_delegate",
        writable,
        docs = "The multi_delegate PDA that will be the delegate instance for this token"
    ))]
    #[codama(account(
        name = "token_mint",
        docs = "The token mint that we are creating a multi delegate account for"
    ))]
    #[codama(account(
        name = "user_ata",
        writable,
        docs = "The ata that we are setting up delegation for"
    ))]
    #[codama(account(
        name = "system_program",
        docs = "The system program",
        default_value = program("system")
    ))]
    #[codama(account(name = "token_program", docs = "Token program"))]
    InitMultiDelegate = 0,

    #[codama(account(
        name = "delegator",
        signer,
        writable,
        docs = "The user creating the delegation"
    ))]
    #[codama(account(
        name = "multi_delegate",
        writable,
        docs = "The multi_delegate PDA for this token"
    ))]
    #[codama(account(
        name = "delegation_account",
        writable,
        docs = "The fixed delegation PDA being created"
    ))]
    #[codama(account(name = "delegatee", docs = "The user receiving delegation rights"))]
    #[codama(account(
        name = "system_program",
        docs = "The system program",
        default_value = program("system")
    ))]
    CreateFixedDelegation(#[codama(name = "fixed_delegation")] CreateFixedDelegationData) = 1,

    #[codama(account(
        name = "delegator",
        signer,
        writable,
        docs = "The user creating the delegation"
    ))]
    #[codama(account(
        name = "multi_delegate",
        writable,
        docs = "The multi_delegate PDA for this token"
    ))]
    #[codama(account(
        name = "delegation_account",
        writable,
        docs = "The recurring delegation PDA being created"
    ))]
    #[codama(account(name = "delegatee", docs = "The user receiving delegation rights"))]
    #[codama(account(
        name = "system_program",
        docs = "The system program",
        default_value = program("system")
    ))]
    CreateRecurringDelegation(
        #[codama(name = "recurring_delegation")] CreateRecurringDelegationData,
    ) = 2,

    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "The delegator revoking the delegation (receives rent)"
    ))]
    #[codama(account(
        name = "delegation_account",
        writable,
        docs = "The delegation PDA to close"
    ))]
    RevokeDelegation = 3,

    #[codama(account(
        name = "delegation_pda",
        writable,
        docs = "The fixed delegation PDA to transfer from"
    ))]
    #[codama(account(name = "multi_delegate", writable, docs = "The multi delegate PDA"))]
    #[codama(account(
        name = "delegator_ata",
        writable,
        docs = "The delegator's ATA to transfer from"
    ))]
    #[codama(account(
        name = "receiver_ata",
        writable,
        docs = "The receiver's ATA to transfer to"
    ))]
    #[codama(account(name = "token_program", docs = "Token program"))]
    #[codama(account(
        name = "delegatee",
        signer,
        docs = "The delegatee signing the transfer"
    ))]
    TransferFixed(#[codama(name = "transfer_data")] TransferData) = 4,

    #[codama(account(
        name = "delegation_pda",
        writable,
        docs = "The recurring delegation PDA to transfer from"
    ))]
    #[codama(account(name = "multi_delegate", writable, docs = "The multi delegate PDA"))]
    #[codama(account(
        name = "delegator_ata",
        writable,
        docs = "The delegator's ATA to transfer from"
    ))]
    #[codama(account(
        name = "receiver_ata",
        writable,
        docs = "The receiver's ATA to transfer to"
    ))]
    #[codama(account(name = "token_program", docs = "Token program"))]
    #[codama(account(
        name = "delegatee",
        signer,
        docs = "The delegatee signing the transfer"
    ))]
    TransferRecurring(#[codama(name = "transfer_data")] TransferData) = 5,

    #[codama(account(
        name = "user",
        signer,
        writable,
        docs = "The user who owns the MultiDelegate PDA (receives rent)"
    ))]
    #[codama(account(
        name = "multi_delegate",
        writable,
        docs = "The MultiDelegate PDA to close"
    ))]
    CloseMultiDelegate = 6,
}

impl MultiDelegatorInstruction {
    /// Parse a `MultiDelegatorInstruction` from raw instruction bytes.
    /// The first byte is the discriminator, followed by instruction-specific data.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        let (discriminator, rest) = data
            .split_first()
            .ok_or(MultiDelegatorError::InvalidInstruction)?;

        match discriminator {
            0 => Ok(Self::InitMultiDelegate),
            1 => {
                let loaded = CreateFixedDelegationData::load(rest)?;
                // Found that this clones to only add 300cus, much lower than expected.
                // Will come back to this to see if we can still keep nice interface plus less CUs
                Ok(Self::CreateFixedDelegation(loaded.clone()))
            }
            2 => {
                let loaded = CreateRecurringDelegationData::load(rest)?;
                Ok(Self::CreateRecurringDelegation(loaded.clone()))
            }
            3 => Ok(Self::RevokeDelegation),
            4 => {
                let loaded = TransferData::load(rest)?;
                Ok(Self::TransferFixed(loaded.clone()))
            }
            5 => {
                let loaded = TransferData::load(rest)?;
                Ok(Self::TransferRecurring(loaded.clone()))
            }
            6 => Ok(Self::CloseMultiDelegate),
            _ => Err(MultiDelegatorError::InvalidInstruction.into()),
        }
    }
}

impl fmt::Display for MultiDelegatorInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitMultiDelegate => write!(f, "init_multi_delegate"),
            Self::CreateFixedDelegation(_) => write!(f, "create_fixed_delegation"),
            Self::CreateRecurringDelegation(_) => write!(f, "create_recurring_delegation"),
            Self::RevokeDelegation => write!(f, "revoke_delegation"),
            Self::TransferFixed(_) => write!(f, "transfer_fixed"),
            Self::TransferRecurring(_) => write!(f, "transfer_recurring"),
            Self::CloseMultiDelegate => write!(f, "close_multi_delegate"),
        }
    }
}
