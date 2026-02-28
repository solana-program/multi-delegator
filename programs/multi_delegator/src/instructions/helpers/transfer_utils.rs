use pinocchio::{
    cpi::{Seed, Signer},
    AccountView, Address, ProgramResult,
};
use pinocchio_token_2022::instructions::Transfer;

use crate::{
    constants::{
        TOKEN_ACCOUNT_MINT_END, TOKEN_ACCOUNT_MINT_OFFSET, TOKEN_ACCOUNT_OWNER_END,
        TOKEN_ACCOUNT_OWNER_OFFSET,
    },
    MultiDelegate, MultiDelegatorError,
};

pub fn check_token_account_owner(
    data: &[u8],
    expected: &Address,
) -> Result<(), MultiDelegatorError> {
    if data.len() < TOKEN_ACCOUNT_OWNER_END {
        return Err(MultiDelegatorError::InvalidAccountData);
    }
    if data[TOKEN_ACCOUNT_OWNER_OFFSET..TOKEN_ACCOUNT_OWNER_END] != expected.as_ref()[..] {
        return Err(MultiDelegatorError::Unauthorized);
    }
    Ok(())
}

pub fn check_token_account_mint(
    data: &[u8],
    expected: &Address,
) -> Result<(), MultiDelegatorError> {
    if data.len() < TOKEN_ACCOUNT_MINT_END {
        return Err(MultiDelegatorError::InvalidAccountData);
    }
    if data[TOKEN_ACCOUNT_MINT_OFFSET..TOKEN_ACCOUNT_MINT_END] != expected.as_ref()[..] {
        return Err(MultiDelegatorError::MintMismatch);
    }
    Ok(())
}

pub fn get_token_account_owner(data: &[u8]) -> Result<Address, MultiDelegatorError> {
    if data.len() < TOKEN_ACCOUNT_OWNER_END {
        return Err(MultiDelegatorError::InvalidAccountData);
    }
    let mut owner = [0u8; 32];
    owner.copy_from_slice(&data[TOKEN_ACCOUNT_OWNER_OFFSET..TOKEN_ACCOUNT_OWNER_END]);
    Ok(Address::from(owner))
}

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

    {
        let ata_data = accounts.delegator_ata.try_borrow()?;
        check_token_account_owner(&ata_data, delegator)?;
        check_token_account_mint(&ata_data, mint)?;
    }

    {
        let to_data = accounts.to_ata.try_borrow()?;
        check_token_account_mint(&to_data, mint)?;
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
        token_program: accounts.token_program.address(),
    }
    .invoke_signed(&signer)?;

    Ok(())
}
