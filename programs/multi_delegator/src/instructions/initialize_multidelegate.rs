use pinocchio::{cpi::Seed, error::ProgramError, AccountView, ProgramResult};

use pinocchio_token::instructions::Approve as ApproveSpl;
use pinocchio_token_2022::instructions::Approve as Approve2022;

use crate::{
    constants::TOKEN_2022_PROGRAM_ID, AccountCheck, MintInterface, MultiDelegate,
    MultiDelegatorError, ProgramAccount, ProgramAccountInit, SignerAccount, SystemAccount,
    TokenAccountInterface, TokenProgramInterface,
};

pub struct InitializeMultiDelegateAccounts<'a> {
    pub user: &'a AccountView,
    pub multi_delegate: &'a AccountView,
    pub token_mint: &'a AccountView,
    pub user_ata: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for InitializeMultiDelegateAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
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

pub fn process(accounts: &[AccountView]) -> ProgramResult {
    let accounts = InitializeMultiDelegateAccounts::try_from(accounts)?;

    let (expected_pda, bump) =
        MultiDelegate::find_pda(accounts.user.address(), accounts.token_mint.address());

    if expected_pda != *accounts.multi_delegate.address() {
        return Err(MultiDelegatorError::InvalidMultiDelegatePda.into());
    }

    let bump_binding = [bump];
    let seeds = [
        Seed::from(MultiDelegate::SEED),
        Seed::from(accounts.user.address().as_ref()),
        Seed::from(accounts.token_mint.address().as_ref()),
        Seed::from(&bump_binding),
    ];

    // Initialize the account if it doesn't exist
    if accounts.multi_delegate.data_len() == 0 {
        ProgramAccount::init::<MultiDelegate>(
            accounts.user,
            accounts.multi_delegate,
            &seeds,
            MultiDelegate::LEN,
        )?;

        let mut data = accounts.multi_delegate.try_borrow_mut()?;
        MultiDelegate::init(
            &mut data,
            accounts.user.address(),
            accounts.token_mint.address(),
            bump,
        )?;
    }

    // Approve delegation on the correct token program (SPL Token vs Token-2022).
    // The instruction data is the same, but the program id differs.
    if accounts.token_program.address().eq(&TOKEN_2022_PROGRAM_ID) {
        Approve2022 {
            token_program: accounts.token_program.address(),
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
        AccountDiscriminator, MultiDelegate,
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

        assert_eq!(
            multi_delegate.discriminator,
            AccountDiscriminator::MultiDelegate as u8
        );
        assert_eq!(multi_delegate.user.to_bytes(), user.pubkey().to_bytes());
        assert_eq!(multi_delegate.token_mint.to_bytes(), mint.to_bytes());
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

        assert_eq!(
            multi_delegate.discriminator,
            AccountDiscriminator::MultiDelegate as u8
        );
        assert_eq!(multi_delegate.user.to_bytes(), user.pubkey().to_bytes());
        assert_eq!(multi_delegate.token_mint.to_bytes(), mint.to_bytes());
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

    /// Verify that pre-funding a MultiDelegate PDA with lamports (DOS attack)
    /// does not prevent the legitimate user from creating the account.
    #[test]
    fn initialize_multidelegate_with_prefunded_pda() {
        use solana_account::Account;

        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
        );
        let user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        // Simulate an attacker pre-funding the PDA address with lamports
        let (multi_delegate_pda, _) =
            crate::tests::pda::get_multidelegate_pda(&user.pubkey(), &mint);
        litesvm
            .set_account(
                multi_delegate_pda,
                Account {
                    lamports: 1_000,
                    data: vec![],
                    owner: solana_pubkey::Pubkey::default(), // system program
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

        // The user should still be able to initialize the multidelegate PDA
        let (res, _, bump) = initialize_multidelegate_action(litesvm, user, mint);
        res.assert_ok();

        let account = litesvm.get_account(&multi_delegate_pda).unwrap();
        let multi_delegate = MultiDelegate::load(&account.data).unwrap();

        assert_eq!(
            multi_delegate.discriminator,
            AccountDiscriminator::MultiDelegate as u8
        );
        assert_eq!(multi_delegate.user.to_bytes(), user.pubkey().to_bytes());
        assert_eq!(multi_delegate.token_mint.to_bytes(), mint.to_bytes());
        assert_eq!(multi_delegate.bump, bump);

        // Verify delegation
        let ata_account = fetch_account::<spl_token_2022::state::Account>(litesvm, &user_ata);
        assert!(ata_account.delegate.is_some());
        assert_eq!(ata_account.delegate.unwrap(), multi_delegate_pda);
        assert_eq!(ata_account.delegated_amount, u64::MAX);
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
