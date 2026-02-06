use pinocchio::program_error::ProgramError;
use thiserror::Error;

impl From<MultiDelegatorError> for ProgramError {
    fn from(e: MultiDelegatorError) -> Self {
        ProgramError::Custom(e as u32)
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
    InvalidPayerData,
}
