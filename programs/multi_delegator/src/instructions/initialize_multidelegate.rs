use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Approve as ApproveSpl;
use pinocchio_token_2022::instructions::Approve as Approve2022;

use crate::{
    constants::TOKEN_2022_PROGRAM_ID, AccountCheck, MintInterface, MultiDelegate,
    MultiDelegatorError, SignerAccount, SystemAccount, TokenAccountInterface,
    TokenProgramInterface,
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
        let [user, multi_delegate, token_mint, user_ata, system_program, token_program] = accounts
        else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(user)?;
        MintInterface::check_with_program(token_mint, token_program)?;
        TokenAccountInterface::check_with_program(user_ata, token_program)?;
        TokenProgramInterface::check(token_program)?;
        SystemAccount::check(system_program)?;

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

pub fn process(accounts: &[AccountInfo]) -> ProgramResult {
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

    // Approve delegation on the correct token program (SPL Token vs Token-2022).
    // The instruction data is the same, but the program id differs.
    if accounts.token_program.key().as_ref() == TOKEN_2022_PROGRAM_ID {
        Approve2022 {
            token_program: accounts.token_program.key(),
            source: accounts.user_ata,
            delegate: accounts.multi_delegate,
            authority: accounts.user,
            amount: u64::MAX,
        }
        .invoke()?;
    } else {
        ApproveSpl {
            source: accounts.user_ata,
            delegate: accounts.multi_delegate,
            authority: accounts.user,
            amount: u64::MAX,
        }
        .invoke()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use solana_signer::Signer;

    use crate::{
        tests::{
            asserts::TransactionResultExt,
            constants::{
                MINT_DECIMALS, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
            },
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
        res.assert_ok();

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

    #[test]
    fn initialize_multidelegate_token_2022() {
        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_2022_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
        );
        let user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (res, multi_delegate_pda, bump) = initialize_multidelegate_action(litesvm, user, mint);
        res.assert_ok();

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

    #[test]
    fn wrong_token_program_returns_error() {
        use solana_instruction::{AccountMeta, Instruction};
        use solana_signer::Signer;

        use crate::{
            instructions::initialize_multidelegate,
            tests::{
                constants::PROGRAM_ID, constants::SYSTEM_PROGRAM_ID, pda::get_multidelegate_pda,
                utils::build_and_send_transaction,
            },
        };

        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
        );
        let user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (multi_delegate_pda, _bump) = get_multidelegate_pda(&user.pubkey(), &mint);

        let fake_token_program = user.pubkey();

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(multi_delegate_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(user_ata, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(fake_token_program, false),
            ],
            data: vec![*initialize_multidelegate::DISCRIMINATOR],
        };

        let res = build_and_send_transaction(litesvm, &[user], &user.pubkey(), &ix);
        assert!(res.is_err());
    }

    #[test]
    fn extra_accounts_rejected() {
        use solana_instruction::{AccountMeta, Instruction};
        use solana_signer::Signer;

        use crate::{
            instructions::initialize_multidelegate,
            tests::{
                constants::PROGRAM_ID, constants::TOKEN_PROGRAM_ID, pda::get_multidelegate_pda,
                utils::build_and_send_transaction,
            },
        };

        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
        );
        let user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (multi_delegate_pda, _bump) = get_multidelegate_pda(&user.pubkey(), &mint);

        let extra_account = user.pubkey();

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(multi_delegate_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(user_ata, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(extra_account, false),
            ],
            data: vec![*initialize_multidelegate::DISCRIMINATOR],
        };

        let res = build_and_send_transaction(litesvm, &[user], &user.pubkey(), &ix);
        assert!(res.is_err());
    }
}
