use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

/// AccountCheck is a trait that is used to signify some sort of evaluation on an account
/// For Example for MintAccount, the check makes sure that the token belongs to the tokens program
/// and has the correct size.
pub trait AccountCheck {
    fn check(account: &AccountView) -> Result<(), ProgramError>;
}

pub trait MintInit {
    fn init(
        account: &AccountView,
        payer: &AccountView,
        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult;
    fn init_if_needed(
        account: &AccountView,
        payer: &AccountView,
        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult;
}

pub trait TokenInit {
    fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &Address,
    ) -> ProgramResult;
    fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &Address,
    ) -> ProgramResult;
}

pub trait AssociatedTokenAccountCheck {
    fn check(
        account: &AccountView,
        authority: &AccountView,
        mint: &AccountView,
        token_program: &AccountView,
    ) -> Result<(), ProgramError>;
}

pub trait AssociatedTokenAccountInit {
    fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult;
    fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult;
}

pub trait ProgramAccountInit {
    fn init<'a, T: Sized>(
        payer: &AccountView,
        account: &AccountView,
        seeds: &[Seed<'a>],
        space: usize,
    ) -> ProgramResult;
}

pub trait AccountClose {
    fn close(account: &AccountView, destination: &AccountView) -> ProgramResult;
}
