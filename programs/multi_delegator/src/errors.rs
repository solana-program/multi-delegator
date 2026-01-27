use pinocchio::program_error::ProgramError;

impl From<MultiDelegatorError> for ProgramError {
    fn from(e: MultiDelegatorError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

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
}
