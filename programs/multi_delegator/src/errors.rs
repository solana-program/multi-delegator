use pinocchio::error::ProgramError;
use thiserror::Error;

impl From<MultiDelegatorError> for ProgramError {
    fn from(e: MultiDelegatorError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[cfg(test)]
impl TryFrom<u32> for MultiDelegatorError {
    type Error = u32;

    fn try_from(code: u32) -> Result<Self, Self::Error> {
        match code {
            0 => Ok(Self::NotSigner),
            1 => Ok(Self::InvalidAddress),
            2 => Ok(Self::InvalidEscrowPda),
            3 => Ok(Self::InvalidMultiDelegatePda),
            4 => Ok(Self::NotSystemProgram),
            5 => Ok(Self::InvalidTokenProgram),
            6 => Ok(Self::InvalidToken2022MintAccountData),
            7 => Ok(Self::InvalidToken2022TokenAccountData),
            8 => Ok(Self::InvalidAssociatedTokenAccountDerivedAddress),
            9 => Ok(Self::InvalidTokenSplMintAccountData),
            10 => Ok(Self::InvalidTokenSplTokenAccountData),
            11 => Ok(Self::InvalidDelegatePda),
            12 => Ok(Self::InvalidDelegationKind),
            13 => Ok(Self::InvalidAccountData),
            14 => Ok(Self::InvalidHeaderData),
            15 => Ok(Self::InvalidInstructionData),
            16 => Ok(Self::NotEnoughAccountKeys),
            17 => Ok(Self::InvalidInstruction),
            18 => Ok(Self::DelegationExpired),
            19 => Ok(Self::AmountExceedsLimit),
            20 => Ok(Self::AmountExceedsPeriodLimit),
            21 => Ok(Self::PeriodNotElapsed),
            22 => Ok(Self::TimestampError),
            23 => Ok(Self::InvalidAmount),
            24 => Ok(Self::TransferInvalidKind),
            25 => Ok(Self::TransferInvalidCallData),
            26 => Ok(Self::Unauthorized),
            27 => Ok(Self::TransferKindMismatch),
            28 => Ok(Self::ArithmeticOverflow),
            29 => Ok(Self::InvalidPeriodLength),
            30 => Ok(Self::InvalidPayerData),
            31 => Ok(Self::ArithmeticUnderflow),
            _ => Err(code),
        }
    }
}

#[derive(Debug, Copy, Clone, Error)]
pub enum MultiDelegatorError {
    #[error("Account must be a signer")]
    NotSigner,
    #[error("Invalid account address")]
    InvalidAddress,
    #[error("Invalid escrow PDA derivation")]
    InvalidEscrowPda,
    #[error("Invalid multi-delegate PDA derivation")]
    InvalidMultiDelegatePda,
    #[error("Expected system program")]
    NotSystemProgram,
    #[error("Token Program does not match other accounts")]
    InvalidTokenProgram,
    #[error("Invalid Token-2022 mint account data")]
    InvalidToken2022MintAccountData,
    #[error("Invalid Token-2022 token account data")]
    InvalidToken2022TokenAccountData,
    #[error("Invalid associated token account address")]
    InvalidAssociatedTokenAccountDerivedAddress,
    #[error("Invalid SPL Token mint account data")]
    InvalidTokenSplMintAccountData,
    #[error("Invalid SPL Token account data")]
    InvalidTokenSplTokenAccountData,
    #[error("Invalid delegation PDA derivation")]
    InvalidDelegatePda,
    #[error("Invalid delegation kind")]
    InvalidDelegationKind,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Invalid header data")]
    InvalidHeaderData,
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    #[error("Not enough account keys provided")]
    NotEnoughAccountKeys,
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Delegation has expired")]
    DelegationExpired,
    #[error("Transfer amount exceeds delegation limit")]
    AmountExceedsLimit,
    #[error("Transfer amount exceeds period limit")]
    AmountExceedsPeriodLimit,
    #[error("Period has not elapsed yet")]
    PeriodNotElapsed,
    #[error("Clock timestamp error")]
    TimestampError,
    #[error("Invalid amount specified")]
    InvalidAmount,
    #[error("Invalid transfer kind")]
    TransferInvalidKind,
    #[error("Invalid transfer call data")]
    TransferInvalidCallData,
    #[error("Caller not authorized for this action")]
    Unauthorized,
    #[error("Transfer kind does not match delegation type")]
    TransferKindMismatch,
    #[error("Arithmetic Overflow")]
    ArithmeticOverflow,
    #[error("Invalid Period length")]
    InvalidPeriodLength,
    #[error("Payer provided does not match delegation")]
    InvalidPayerData,
    #[error("Arithmetic Underflow")]
    ArithmeticUnderflow,
}
