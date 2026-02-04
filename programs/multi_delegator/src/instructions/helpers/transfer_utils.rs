use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_token_2022::instructions::Transfer;

use crate::{MultiDelegate, MultiDelegatorError};

pub struct TransferAccounts<'a> {
    pub delegator_ata: &'a AccountInfo,
    /// Account to whom these funds are being transfered too
    pub to_ata: &'a AccountInfo,
    /// Pda that is the delegate to the delegators tokens
    pub multidelegate_pda: &'a AccountInfo,
    /// The token program (SPL Token or Token-2022)
    pub token_program: &'a AccountInfo,
}

pub fn transfer_with_delegate(
    amount: u64,
    delegator: &Pubkey,
    mint: &Pubkey,
    accounts: &TransferAccounts,
) -> ProgramResult {
    let (expected_pda, bump) = MultiDelegate::find_pda(delegator, mint);

    if expected_pda != *accounts.multidelegate_pda.key() {
        return Err(MultiDelegatorError::InvalidDelegatePda.into());
    }

    let bump_bytes = [bump];
    let seeds = [
        Seed::from(MultiDelegate::SEED),
        Seed::from(delegator.as_ref()),
        Seed::from(mint.as_ref()),
        Seed::from(&bump_bytes),
    ];
    let signer = [Signer::from(&seeds)];

    Transfer {
        from: accounts.delegator_ata,
        to: accounts.to_ata,
        authority: accounts.multidelegate_pda,
        amount,
        token_program: accounts.token_program.key(),
    }
    .invoke_signed(&signer)?;

    Ok(())
}
