use codama::CodamaType;
use pinocchio::error::ProgramError;
use pinocchio::Address;

use crate::MultiDelegatorError;

pub const DELEGATE_BASE_SEED: &[u8] = b"delegation";

pub fn verify_delegation_pda(
    multi_delegate: &Address,
    delegator: &Address,
    delegatee: &Address,
    nonce: u64,
    bump: u8,
) -> Result<Address, ProgramError> {
    Address::create_program_address(
        &[
            DELEGATE_BASE_SEED,
            multi_delegate.as_ref(),
            delegator.as_ref(),
            delegatee.as_ref(),
            &nonce.to_le_bytes(),
            &[bump],
        ],
        &crate::ID,
    )
    .map_err(|_| MultiDelegatorError::InvalidDelegatePda.into())
}

pub fn find_delegation_pda(
    multi_delegate: &Address,
    delegator: &Address,
    delegatee: &Address,
    nonce: u64,
) -> (Address, u8) {
    Address::find_program_address(
        &[
            DELEGATE_BASE_SEED,
            multi_delegate.as_ref(),
            delegator.as_ref(),
            delegatee.as_ref(),
            &nonce.to_le_bytes(),
        ],
        &crate::ID,
    )
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug, CodamaType)]
pub enum AccountDiscriminator {
    MultiDelegate = 0,
    Plan = 1,
    FixedDelegation = 2,
    RecurringDelegation = 3,
    SubscriptionDelegation = 4,
}

impl TryFrom<u8> for AccountDiscriminator {
    type Error = ProgramError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MultiDelegate),
            1 => Ok(Self::Plan),
            2 => Ok(Self::FixedDelegation),
            3 => Ok(Self::RecurringDelegation),
            4 => Ok(Self::SubscriptionDelegation),
            _ => Err(MultiDelegatorError::InvalidAccountDiscriminator.into()),
        }
    }
}

impl From<AccountDiscriminator> for u8 {
    fn from(val: AccountDiscriminator) -> Self {
        match val {
            AccountDiscriminator::MultiDelegate => 0,
            AccountDiscriminator::Plan => 1,
            AccountDiscriminator::FixedDelegation => 2,
            AccountDiscriminator::RecurringDelegation => 3,
            AccountDiscriminator::SubscriptionDelegation => 4,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug, CodamaType)]
pub enum PlanStatus {
    Sunset = 0,
    Active = 1,
}

impl TryFrom<u8> for PlanStatus {
    type Error = ProgramError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Sunset),
            1 => Ok(Self::Active),
            _ => Err(MultiDelegatorError::InvalidAccountData.into()),
        }
    }
}

pub fn find_plan_pda(owner: &Address, plan_id: u64) -> (Address, u8) {
    Address::find_program_address(
        &[
            crate::state::plan::Plan::SEED,
            owner.as_ref(),
            &plan_id.to_le_bytes(),
        ],
        &crate::ID,
    )
}

pub fn find_subscription_pda(plan_pda: &Address, subscriber: &Address) -> (Address, u8) {
    Address::find_program_address(
        &[
            crate::state::subscription_delegation::SubscriptionDelegation::SEED,
            plan_pda.as_ref(),
            subscriber.as_ref(),
        ],
        &crate::ID,
    )
}
