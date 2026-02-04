use pinocchio::program_error::ProgramError;

impl From<MultiDelegatorError> for ProgramError {
    fn from(e: MultiDelegatorError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MultiDelegatorError {
    NotSigner,
    InvalidAddress,
    InvalidEscrowPda,
    InvalidMultiDelegatePda,
    NotSystemProgram,
    InvalidToken2022MintAccountData,
    InvalidToken2022TokenAccountData,
    InvalidAssociatedTokenAccountDerivedAddress,
    InvalidTokenSplMintAccountData,
    InvalidTokenSplTokenAccountData,
    InvalidDelegatePda,
    InvalidDelegationKind,
    InvalidAccountData,
    InvalidHeaderData,
    InvalidInstructionData,
    NotEnoughAccountKeys,
    InvalidInstruction,
    DelegationExpired,
    AmountExceedsLimit,
    AmountExceedsPeriodLimit,
    PeriodNotElapsed,
    TimestampError,
    InvalidAmount,
    TransferInvalidKind,
    TransferInvalidCallData,
    Unauthorized,
    TransferKindMismatch,
}
