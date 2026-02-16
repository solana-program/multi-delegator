use pinocchio::{
    cpi::{Seed, Signer},
    AccountView, Address, ProgramResult,
};
use pinocchio_token_2022::instructions::Transfer;

use crate::{MultiDelegate, MultiDelegatorError};

pub struct TransferAccounts<'a> {
    pub delegator_ata: &'a AccountView,
    /// Account to whom these funds are being transfered too
    pub to_ata: &'a AccountView,
    /// Pda that is the delegate to the delegators tokens
    pub multidelegate_pda: &'a AccountView,
    /// The token program (SPL Token or Token-2022)
    pub token_program: &'a AccountView,
}

pub fn transfer_with_delegate(
    amount: u64,
    delegator: &Address,
    mint: &Address,
    accounts: &TransferAccounts,
) -> ProgramResult {
    let bump = {
        // Read the bump from the MultiDelegate account data (cheaper than find_program_address)
        let multidelegate_data = accounts.multidelegate_pda.try_borrow()?;
        let multidelegate = MultiDelegate::load(&multidelegate_data)?;

        // Verify that the MultiDelegate account matches the provided delegator and mint.
        // Since the account is owned by the program (checked in instruction processor),
        // we can trust its data. If the data matches, it is the correct PDA.
        if multidelegate.user != *delegator || multidelegate.token_mint != *mint {
            return Err(MultiDelegatorError::InvalidDelegatePda.into());
        }
        multidelegate.bump
    };

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
        token_program: accounts.token_program.address(),
    }
    .invoke_signed(&signer)?;

    Ok(())
}
