use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Approve;

use crate::{
    AccountCheck, MintInterface, MultiDelegate, MultiDelegatorError, SignerAccount, SystemAccount,
    TokenAccountInterface,
};

pub struct InitializeMultiDelegateAccounts<'a> {
    pub user: &'a AccountInfo,
    pub multi_delegate: &'a AccountInfo,
    pub token_mint: &'a AccountInfo,
    pub user_ata: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeMultiDelegateAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // We use .. to allow for extra accounts if any (though usually strict is better, debugging hints at potential issues)
        let [user, multi_delegate, token_mint, user_ata, system_program, token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(user)?;
        MintInterface::check(token_mint)?;
        TokenAccountInterface::check(user_ata)?;
        msg!("Before system check");
        SystemAccount::check(system_program)?;
        msg!("After system check");
        // TODO produce check that can verify that the user_ata, token_mint are linked too by token_program
        assert_eq!(user_ata.owner(), token_program.key());
        assert_eq!(token_mint.owner(), token_program.key());

        Ok(Self {
            multi_delegate,
            user,
            token_mint,
            user_ata,
            system_program,
            token_program,
        })
    }
}

pub const DISCRIMINATOR: &u8 = &0;

pub fn process((_data, accounts): (&[u8], &[AccountInfo])) -> ProgramResult {
    let accounts = InitializeMultiDelegateAccounts::try_from(accounts)?;

    let (expected_pda, bump) =
        MultiDelegate::find_pda(accounts.user.key(), accounts.token_mint.key());

    if expected_pda != *accounts.multi_delegate.key() {
        return Err(MultiDelegatorError::InvalidMultiDelegatePda.into());
    }

    let bump_binding = [bump];
    let seeds = [
        Seed::from(MultiDelegate::SEED),
        Seed::from(accounts.user.key().as_ref()),
        Seed::from(accounts.token_mint.key().as_ref()),
        Seed::from(&bump_binding),
    ];

    // Initialize the account
    if accounts.multi_delegate.data_len() == 0 {
        let lamports = Rent::get()?.minimum_balance(MultiDelegate::LEN);
        let signer = [Signer::from(&seeds)];

        CreateAccount {
            from: accounts.user,
            to: accounts.multi_delegate,
            lamports,
            space: MultiDelegate::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signer)?;

        let mut data = accounts.multi_delegate.try_borrow_mut_data()?;
        let multi_delegate_state = MultiDelegate::load_mut(&mut data)?;

        multi_delegate_state.user = *accounts.user.key();
        multi_delegate_state.token_mint = *accounts.token_mint.key();
        multi_delegate_state.bump = bump;
    }

    Approve {
        source: accounts.user_ata,
        delegate: accounts.multi_delegate,
        authority: accounts.user,
        amount: u64::MAX,
    }
    .invoke()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use solana_signer::Signer;

    use crate::{
        tests::{
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{fetch_account, init_ata, init_mint, initialize_multidelegate_action, setup},
        },
        MultiDelegate,
    };

    #[test]
    fn initialize_multidelegate() {
        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
        );
        let user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (res, multi_delegate_pda, bump) = initialize_multidelegate_action(litesvm, user, mint);
        res.unwrap();

        let account = litesvm.get_account(&multi_delegate_pda).unwrap();
        let multi_delegate = MultiDelegate::load(&account.data).unwrap();

        assert_eq!(multi_delegate.user, user.pubkey().to_bytes());
        assert_eq!(multi_delegate.token_mint, mint.to_bytes());
        assert_eq!(multi_delegate.bump, bump);

        // Verify delegation
        let ata_account = fetch_account::<spl_token_2022::state::Account>(litesvm, &user_ata);
        assert!(ata_account.delegate.is_some());
        assert_eq!(ata_account.delegate.unwrap(), multi_delegate_pda);
        assert_eq!(ata_account.delegated_amount, u64::MAX);
    }
}
