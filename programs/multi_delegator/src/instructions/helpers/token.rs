use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{
    instructions::{InitializeAccount3, InitializeMint2},
    state::{Mint, TokenAccount as TokenAccountState},
    ID as SPL_TOKEN_PROGRAM_ID,
};

use super::traits::{
    AccountCheck, AssociatedTokenAccountCheck, AssociatedTokenAccountInit, MintInit, TokenInit,
};
use crate::{
    constants::{
        TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET, TOKEN_2022_MINT_DISCRIMINATOR,
        TOKEN_2022_PROGRAM_ID, TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR,
    },
    MultiDelegatorError,
};

// Private helpers to consolidate initialization logic

fn init_mint_helper(
    account: &AccountInfo,
    payer: &AccountInfo,
    decimals: u8,
    mint_authority: &[u8; 32],
    freeze_authority: Option<&[u8; 32]>,
    owner_program_id: &Pubkey,
) -> ProgramResult {
    let lamports = Rent::get()?.minimum_balance(Mint::LEN);

    CreateAccount {
        from: payer,
        to: account,
        lamports,
        space: Mint::LEN as u64,
        owner: owner_program_id,
    }
    .invoke()?;

    InitializeMint2 {
        mint: account,
        decimals,
        mint_authority,
        freeze_authority,
    }
    .invoke()
}

fn init_token_helper(
    account: &AccountInfo,
    mint: &AccountInfo,
    payer: &AccountInfo,
    owner: &[u8; 32],
    owner_program_id: &Pubkey,
) -> ProgramResult {
    let lamports = Rent::get()?.minimum_balance(TokenAccountState::LEN);

    CreateAccount {
        from: payer,
        to: account,
        lamports,
        space: TokenAccountState::LEN as u64,
        owner: owner_program_id,
    }
    .invoke()?;

    InitializeAccount3 {
        account,
        mint,
        owner,
    }
    .invoke()
}

// MintAccount (SPL Token)

pub struct MintAccount;

impl AccountCheck for MintAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().ne(&pinocchio_token::ID) {
            return Err(MultiDelegatorError::InvalidTokenSplMintAccountData.into());
        }

        if account.data_len() != Mint::LEN {
            return Err(MultiDelegatorError::InvalidTokenSplMintAccountData.into());
        }

        Ok(())
    }
}

impl MintInit for MintAccount {
    fn init(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>,
    ) -> ProgramResult {
        init_mint_helper(
            account,
            payer,
            decimals,
            mint_authority,
            freeze_authority,
            &pinocchio_token::ID,
        )
    }

    fn init_if_needed(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, payer, decimals, mint_authority, freeze_authority),
        }
    }
}

// TokenAccount (SPL Token)

pub struct TokenAccount;

impl AccountCheck for TokenAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().ne(&pinocchio_token::ID) {
            return Err(MultiDelegatorError::InvalidTokenSplTokenAccountData.into());
        }

        if account.data_len().ne(&TokenAccountState::LEN) {
            return Err(MultiDelegatorError::InvalidTokenSplTokenAccountData.into());
        }

        Ok(())
    }
}

impl TokenInit for TokenAccount {
    fn init(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &[u8; 32],
    ) -> ProgramResult {
        init_token_helper(account, mint, payer, owner, &pinocchio_token::ID)
    }

    fn init_if_needed(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &[u8; 32],
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, owner),
        }
    }
}

// Mint2022Account

pub struct Mint2022Account;

impl AccountCheck for Mint2022Account {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().ne(&crate::constants::TOKEN_2022_PROGRAM_ID) {
            return Err(MultiDelegatorError::InvalidToken2022MintAccountData.into());
        }

        let data = account.try_borrow_data()?;

        if data.len().ne(&Mint::LEN)
            && data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&TOKEN_2022_MINT_DISCRIMINATOR)
        {
            return Err(MultiDelegatorError::InvalidToken2022MintAccountData.into());
        }

        Ok(())
    }
}

impl MintInit for Mint2022Account {
    fn init(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>,
    ) -> ProgramResult {
        init_mint_helper(
            account,
            payer,
            decimals,
            mint_authority,
            freeze_authority,
            &crate::constants::TOKEN_2022_PROGRAM_ID, // Pass as ref to array which is Pubkey
        )
    }

    fn init_if_needed(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, payer, decimals, mint_authority, freeze_authority),
        }
    }
}

// TokenAccount2022Account

pub struct TokenAccount2022Account;

impl AccountCheck for TokenAccount2022Account {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().ne(&TOKEN_2022_PROGRAM_ID) {
            return Err(MultiDelegatorError::InvalidToken2022TokenAccountData.into());
        }

        let data = account.try_borrow_data()?;

        if data.len().ne(&TokenAccountState::LEN)
            && data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET]
                .ne(&TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR)
        {
            return Err(MultiDelegatorError::InvalidToken2022TokenAccountData.into());
        }

        Ok(())
    }
}

impl TokenInit for TokenAccount2022Account {
    fn init(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &[u8; 32],
    ) -> ProgramResult {
        init_token_helper(
            account,
            mint,
            payer,
            owner,
            &crate::constants::TOKEN_2022_PROGRAM_ID,
        )
    }

    fn init_if_needed(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &[u8; 32],
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, owner),
        }
    }
}

// Interfaces

pub struct TokenProgramInterface;

impl TokenProgramInterface {
    pub fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.key().ne(&SPL_TOKEN_PROGRAM_ID) && account.key().ne(&TOKEN_2022_PROGRAM_ID) {
            return Err(MultiDelegatorError::InvalidTokenProgram.into());
        }
        Ok(())
    }
}

pub struct MintInterface;

impl AccountCheck for MintInterface {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().eq(&TOKEN_2022_PROGRAM_ID) {
            Mint2022Account::check(account)
        } else {
            MintAccount::check(account)
        }
    }
}

impl MintInterface {
    pub fn check_with_program(
        account: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError> {
        Self::check(account)?;

        if account.owner().ne(token_program.key()) {
            return Err(MultiDelegatorError::InvalidTokenProgram.into());
        }

        Ok(())
    }
}

pub struct TokenAccountInterface;

impl AccountCheck for TokenAccountInterface {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().eq(&TOKEN_2022_PROGRAM_ID) {
            TokenAccount2022Account::check(account)
        } else {
            TokenAccount::check(account)
        }
    }
}

impl TokenAccountInterface {
    pub fn check_with_program(
        account: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError> {
        Self::check(account)?;

        if account.owner().ne(token_program.key()) {
            return Err(MultiDelegatorError::InvalidTokenProgram.into());
        }

        Ok(())
    }

    pub fn check_accounts_with_program(
        token_program: &AccountInfo,
        accounts: &[&AccountInfo],
    ) -> Result<(), ProgramError> {
        for account in accounts {
            Self::check(account)?;

            if account.owner().ne(token_program.key()) {
                return Err(MultiDelegatorError::InvalidTokenProgram.into());
            }
        }
        Ok(())
    }
}

pub struct AssociatedTokenAccount;

impl AssociatedTokenAccountCheck for AssociatedTokenAccount {
    fn check(
        account: &AccountInfo,
        authority: &AccountInfo,
        mint: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError> {
        TokenAccountInterface::check(account)?;

        if find_program_address(
            &[authority.key(), token_program.key(), mint.key()],
            &pinocchio_associated_token_account::ID,
        )
        .0
        .ne(account.key())
        {
            return Err(MultiDelegatorError::InvalidAssociatedTokenAccountDerivedAddress.into());
        }

        Ok(())
    }
}

impl AssociatedTokenAccountInit for AssociatedTokenAccount {
    fn init(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult {
        Create {
            funding_account: payer,
            account,
            wallet: owner,
            mint,
            system_program,
            token_program,
        }
        .invoke()
    }

    fn init_if_needed(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult {
        match Self::check(account, owner, mint, token_program) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, owner, system_program, token_program),
        }
    }
}
