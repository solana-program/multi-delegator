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
            12 => Ok(Self::InvalidAccountData),
            13 => Ok(Self::InvalidHeaderData),
            14 => Ok(Self::InvalidInstructionData),
            15 => Ok(Self::NotEnoughAccountKeys),
            16 => Ok(Self::InvalidInstruction),
            17 => Ok(Self::DelegationExpired),
            18 => Ok(Self::AmountExceedsLimit),
            19 => Ok(Self::AmountExceedsPeriodLimit),
            20 => Ok(Self::PeriodNotElapsed),
            21 => Ok(Self::TimestampError),
            22 => Ok(Self::InvalidAmount),
            23 => Ok(Self::TransferInvalidKind),
            24 => Ok(Self::TransferInvalidCallData),
            25 => Ok(Self::Unauthorized),
            26 => Ok(Self::TransferKindMismatch),
            27 => Ok(Self::ArithmeticOverflow),
            28 => Ok(Self::InvalidPeriodLength),
            29 => Ok(Self::InvalidPayerData),
            30 => Ok(Self::ArithmeticUnderflow),
            31 => Ok(Self::InvalidAccountDiscriminator),
            32 => Ok(Self::MintHasConfidentialTransfer),
            33 => Ok(Self::MintHasNonTransferable),
            34 => Ok(Self::MintHasPermanentDelegate),
            35 => Ok(Self::MintHasTransferHook),
            36 => Ok(Self::MintHasTransferFee),
            37 => Ok(Self::MintHasMintCloseAuthority),
            38 => Ok(Self::MintHasPausable),
            39 => Ok(Self::RecurringDelegationStartTimeInPast),
            40 => Ok(Self::RecurringDelegationZeroPeriod),
            41 => Ok(Self::RecurringDelegationStartTimeGreaterThanExpiry),
            42 => Ok(Self::RecurringDelegationAmountZero),
            43 => Ok(Self::FixedDelegationExpiryInPast),
            44 => Ok(Self::FixedDelegationAmountZero),
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
    #[error("Invalid account discriminator")]
    InvalidAccountDiscriminator,
    #[error("Mint has ConfidentialTransfer extension")]
    MintHasConfidentialTransfer,
    #[error("Mint has NonTransferable extension")]
    MintHasNonTransferable,
    #[error("Mint has PermanentDelegate extension")]
    MintHasPermanentDelegate,
    #[error("Mint has TransferHook extension")]
    MintHasTransferHook,
    #[error("Mint has TransferFee extension")]
    MintHasTransferFee,
    #[error("Mint has MintCloseAuthority extension")]
    MintHasMintCloseAuthority,
    #[error("Mint has Pausable extension")]
    MintHasPausable,
    #[error("Past start time specified")]
    RecurringDelegationStartTimeInPast,
    #[error("zero period specified")]
    RecurringDelegationZeroPeriod,
    #[error("start time specified is greater than expiry")]
    RecurringDelegationStartTimeGreaterThanExpiry,
    #[error("zero amount specified")]
    RecurringDelegationAmountZero,
    #[error("Expiry time specified is less than current time")]
    FixedDelegationExpiryInPast,
    #[error("zero amount specified")]
    FixedDelegationAmountZero,
}
