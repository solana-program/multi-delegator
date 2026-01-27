use core::mem::transmute;
use pinocchio::{program_error::ProgramError, pubkey::find_program_address, pubkey::Pubkey};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TermsKind {
    OneTime = 0,
    Recurring = 1,
}

impl TryFrom<u8> for TermsKind {
    type Error = ProgramError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OneTime),
            1 => Ok(Self::Recurring),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TermsState {
    Pending = 0,
    Active = 1,
}

impl TryFrom<u8> for TermsState {
    type Error = ProgramError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Pending),
            1 => Ok(Self::Active),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

impl From<TermsState> for u8 {
    fn from(val: TermsState) -> Self {
        match val {
            TermsState::Pending => 0,
            TermsState::Active => 1,
        }
    }
}


#[repr(C)]
#[derive(Debug)]
pub struct OneTimeTerms {
    pub delegator: Pubkey,
    pub kind: TermsKind,
    pub status: TermsState,
    pub max_amount: u64,
    pub remaining_amount: u64,
    pub expiry_s: u64,
}

pub const DELEGATE_BASE_SEED: &[u8] = b"delegation";

impl OneTimeTerms {
    pub const LEN: usize = std::mem::size_of::<OneTimeTerms>();

    pub fn find_pda(
        multidelegate: &Pubkey,
        delegate: &Pubkey,
        payer: &Pubkey,
        kind: TermsState,
    ) -> (Pubkey, u8) {
        find_program_address(
            &[
                DELEGATE_BASE_SEED,
                multidelegate.as_ref(),
                delegate.as_ref(),
                payer.as_ref(),
                &[kind.into()],
            ],
            &crate::ID,
        )
    }

    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}
