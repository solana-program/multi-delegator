use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, ProgramResult,
};

use crate::{
    check_and_update_version,
    state::{
        common::AccountDiscriminator, fixed_delegation::FixedDelegation,
        recurring_delegation::RecurringDelegation, subscription_delegation::SubscriptionDelegation,
    },
    AccountCheck, AccountClose, Header, MultiDelegatorError, ProgramAccount, SignerAccount,
    WritableAccount, DELEGATEE_OFFSET, DELEGATOR_OFFSET, DISCRIMINATOR_OFFSET, PAYER_OFFSET,
};

/// Validated accounts for the [`RevokeDelegation`](crate::MultiDelegatorInstruction::RevokeDelegation) instruction.
pub struct RevokeDelegationAccounts<'a> {
    /// The delegator revoking the delegation (must be signer + writable; receives rent if self-funded).
    pub authority: &'a AccountView,
    /// The delegation PDA to close.
    pub delegation_account: &'a AccountView,
    /// Optional third-party account to receive rent (required when the original
    /// payer differs from the delegator).
    pub receiver: Option<&'a AccountView>,
}

impl<'a> TryFrom<&'a [AccountView]> for RevokeDelegationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, delegation_account, rem @ ..] = accounts else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(authority)?;
        WritableAccount::check(authority)?;
        WritableAccount::check(delegation_account)?;
        ProgramAccount::check(delegation_account)?;

        Ok(Self {
            authority,
            delegation_account,
            receiver: rem.first(),
        })
    }
}

/// Instruction discriminator byte for `RevokeDelegation`.
pub const DISCRIMINATOR: &u8 = &3;

/// Revokes a delegation by closing the delegation PDA.
/// The rent lamports are returned to the original payer.
///
/// For Fixed/Recurring delegations: the delegator can close at any time;
/// a third-party sponsor (original payer) can close only after expiry.
/// For Subscriptions: requires cancellation first (expires_at_ts != 0) and expiration in the past.
pub fn process(accounts: &[AccountView]) -> ProgramResult {
    let accounts = RevokeDelegationAccounts::try_from(accounts)?;

    let destination = {
        let mut data = accounts.delegation_account.try_borrow_mut()?;

        if data.len() < Header::LEN {
            return Err(MultiDelegatorError::InvalidHeaderData.into());
        }

        let kind = AccountDiscriminator::try_from(data[DISCRIMINATOR_OFFSET])?;

        match kind {
            AccountDiscriminator::SubscriptionDelegation => {
                check_and_update_version(&mut data)?;
                let subscription = SubscriptionDelegation::load_with_min_size(&data)?;

                if subscription.header.delegator != *accounts.authority.address() {
                    return Err(MultiDelegatorError::Unauthorized.into());
                }

                // Subscription must be cancelled (expires_at_ts != 0) and expired
                let current_ts = Clock::get()?.unix_timestamp;
                if subscription.expires_at_ts == 0 || subscription.expires_at_ts > current_ts {
                    return Err(MultiDelegatorError::SubscriptionNotCancelled.into());
                }
            }
            AccountDiscriminator::FixedDelegation | AccountDiscriminator::RecurringDelegation => {
                let is_sponsor = check_is_sponsor(&data, accounts.authority)?;

                // Sponsor can only revoke expired delegations
                if is_sponsor {
                    let expiry_ts = match kind {
                        AccountDiscriminator::FixedDelegation => {
                            FixedDelegation::load_with_min_size(&data)?.expiry_ts
                        }
                        _ => RecurringDelegation::load_with_min_size(&data)?.expiry_ts,
                    };
                    if expiry_ts == 0 {
                        return Err(MultiDelegatorError::Unauthorized.into());
                    }
                    let current_ts = Clock::get()?.unix_timestamp;
                    if expiry_ts > current_ts {
                        return Err(MultiDelegatorError::Unauthorized.into());
                    }
                }
            }
            _ => return Err(MultiDelegatorError::InvalidAccountDiscriminator.into()),
        }

        resolve_destination(&data, &accounts)?
    };

    ProgramAccount::close(accounts.delegation_account, destination)
}

/// Checks whether the caller is the sponsor (payer) rather than the delegator.
/// Returns `Unauthorized` if the caller is neither.
fn check_is_sponsor(data: &[u8], authority: &AccountView) -> Result<bool, ProgramError> {
    let delegator_bytes: &[u8; 32] = data[DELEGATOR_OFFSET..DELEGATEE_OFFSET]
        .try_into()
        .map_err(|_| MultiDelegatorError::InvalidHeaderData)?;

    if delegator_bytes == authority.address().as_ref() {
        return Ok(false);
    }

    let payer_bytes: &[u8; 32] = data[PAYER_OFFSET..PAYER_OFFSET + 32]
        .try_into()
        .map_err(|_| MultiDelegatorError::InvalidPayerData)?;

    if payer_bytes == authority.address().as_ref() {
        return Ok(true);
    }

    Err(MultiDelegatorError::Unauthorized.into())
}

/// Resolves the rent destination from the payer field in the header.
/// Rent always goes back to the original payer: if payer == authority, return
/// authority directly; otherwise require a receiver account matching payer.
fn resolve_destination<'a>(
    data: &[u8],
    accounts: &RevokeDelegationAccounts<'a>,
) -> Result<&'a AccountView, ProgramError> {
    let payer_bytes: &[u8; 32] = data[PAYER_OFFSET..PAYER_OFFSET + 32]
        .try_into()
        .map_err(|_| MultiDelegatorError::InvalidPayerData)?;

    if payer_bytes == accounts.authority.address().as_ref() {
        Ok(accounts.authority)
    } else {
        let receiver = accounts
            .receiver
            .ok_or(MultiDelegatorError::NotEnoughAccountKeys)?;
        WritableAccount::check(receiver)?;
        if receiver.address().as_ref() != payer_bytes {
            return Err(MultiDelegatorError::Unauthorized.into());
        }
        Ok(receiver)
    }
}

#[cfg(test)]
mod tests {
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    use crate::{
        tests::{
            asserts::TransactionResultExt,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                current_ts, days, hours, init_ata, init_mint, init_wallet,
                initialize_multidelegate_action, move_clock_forward, setup,
                setup_with_subscription, CancelSubscription, CreateDelegation, CreateSubscription,
                RevokeDelegation, RevokeSubscription,
            },
        },
        AccountDiscriminator, FixedDelegation, MultiDelegatorError, RecurringDelegation,
    };

    #[test]
    fn revoke_fixed_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        let account_before = litesvm.get_account(&delegation_pda);
        assert!(account_before.is_some());
        let binding = account_before.unwrap();
        let delegation_rent = binding.lamports;
        let delegation = FixedDelegation::load(&binding.data).unwrap();
        assert_eq!(
            delegation.header.discriminator,
            AccountDiscriminator::FixedDelegation as u8
        );

        let delegator_balance_before = litesvm.get_account(&payer.pubkey()).unwrap().lamports;

        let res = RevokeDelegation::new(litesvm, payer, mint, delegatee, nonce).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let delegator_balance_after = litesvm.get_account(&payer.pubkey()).unwrap().lamports;
        assert!(delegator_balance_after > delegator_balance_before);
        assert!(delegator_balance_after >= delegator_balance_before + delegation_rent - 10000);
    }

    #[test]
    fn revoke_recurring_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let epoch = days(1);
        let expiry_ts = current_ts() + days(2) as i64;
        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(100, epoch, current_ts(), expiry_ts);
        res.assert_ok();

        let account_before = litesvm.get_account(&delegation_pda);
        assert!(account_before.is_some());
        let binding = account_before.unwrap();
        let delegation_rent = binding.lamports;
        let delegation = RecurringDelegation::load(&binding.data).unwrap();
        assert_eq!(
            delegation.header.discriminator,
            AccountDiscriminator::RecurringDelegation as u8
        );

        let delegator_balance_before = litesvm.get_account(&payer.pubkey()).unwrap().lamports;

        let res = RevokeDelegation::new(litesvm, payer, mint, delegatee, nonce).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let delegator_balance_after = litesvm.get_account(&payer.pubkey()).unwrap().lamports;
        assert!(delegator_balance_after > delegator_balance_before);
        assert!(delegator_balance_after >= delegator_balance_before + delegation_rent - 10000);
    }

    #[test]
    fn non_delegator_cannot_revoke() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let epoch = days(1);
        let expiry_ts = current_ts() + days(2) as i64;
        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(100, epoch, current_ts(), expiry_ts);
        res.assert_ok();

        let attacker = init_wallet(litesvm, 1_000_000_000);
        let (multi_delegate_pda, _) =
            crate::tests::pda::get_multidelegate_pda(&payer.pubkey(), &mint);
        let res = revoke_delegation_action_with_pda(
            litesvm,
            &attacker,
            delegation_pda,
            multi_delegate_pda,
        );
        assert!(res.is_err());

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(account_after.is_some());
        assert!(account_after.as_ref().map(|a| a.lamports).unwrap_or(0) > 0);
    }

    #[test]
    fn closed_account_is_zeroed() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        let account_before = litesvm.get_account(&delegation_pda);
        let _before_data = account_before.as_ref().unwrap().data.clone();

        let res = RevokeDelegation::new(litesvm, payer, mint, delegatee, nonce).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);

        if let Some(account) = account_after {
            assert!(
                account.data.iter().all(|&byte| byte == 0),
                "All data should be zeroed after close"
            );
        }
    }

    #[test]
    fn revoke_with_wrong_receiver_returns_unauthorized() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);
        let wrong_receiver = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, _) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        let result = RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .receiver(wrong_receiver.pubkey())
            .execute();

        result.assert_err(MultiDelegatorError::Unauthorized);
    }

    #[test]
    fn writable_accounts_must_be_writable() {
        use solana_instruction::{AccountMeta, Instruction};

        use crate::{
            instructions::revoke_delegation,
            tests::{
                constants::PROGRAM_ID,
                idl,
                utils::{build_and_send_transaction, init_wallet},
            },
        };

        let writable = idl::writable_account_indices("revokeDelegation");

        let (litesvm, user) = &mut setup();
        let payer = user;
        let fee_payer = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        for (idx, _name, is_signer) in &writable {
            let mut accounts = vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(delegation_pda, false),
            ];

            // Flip writable account to readonly, preserving signer flag
            let pubkey = accounts[*idx].pubkey;
            accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

            let ix = Instruction {
                program_id: PROGRAM_ID,
                accounts,
                data: vec![*revoke_delegation::DISCRIMINATOR],
            };

            let res =
                build_and_send_transaction(litesvm, &[&fee_payer, payer], &fee_payer.pubkey(), &ix);
            res.assert_err(MultiDelegatorError::AccountNotWritable);
        }
    }

    #[test]
    fn signer_accounts_must_be_signers() {
        use solana_instruction::{AccountMeta, Instruction};

        use crate::{
            instructions::revoke_delegation,
            tests::{
                constants::PROGRAM_ID,
                idl,
                utils::{build_and_send_transaction, init_wallet},
            },
        };

        let signers = idl::signer_account_indices("revokeDelegation");

        let (litesvm, user) = &mut setup();
        let payer = user;
        let fee_payer = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        for (idx, _name, is_writable) in &signers {
            let mut accounts = vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(delegation_pda, false),
            ];

            // Flip signer to non-signer, preserving writable flag
            let pubkey = accounts[*idx].pubkey;
            accounts[*idx] = if *is_writable {
                AccountMeta::new(pubkey, false)
            } else {
                AccountMeta::new_readonly(pubkey, false)
            };

            let ix = Instruction {
                program_id: PROGRAM_ID,
                accounts,
                data: vec![*revoke_delegation::DISCRIMINATOR],
            };

            let res = build_and_send_transaction(litesvm, &[&fee_payer], &fee_payer.pubkey(), &ix);
            res.assert_err(MultiDelegatorError::NotSigner);
        }
    }

    #[test]
    fn revoke_subscription_without_cancel_rejected() {
        let (mut litesvm, alice, _merchant, _mint, _plan_pda, _, subscription_pda) =
            setup_with_subscription();

        // Try to revoke without cancelling first
        let result = RevokeSubscription::new(&mut litesvm, &alice, subscription_pda).execute();
        result.assert_err(MultiDelegatorError::SubscriptionNotCancelled);

        // Account should still exist
        let account = litesvm.get_account(&subscription_pda);
        assert!(account.is_some());
    }

    #[test]
    fn revoke_subscription_after_cancel_succeeds() {
        let (mut litesvm, alice, _merchant, _mint, plan_pda, _plan_bump, subscription_pda) =
            setup_with_subscription();

        let balance_before = litesvm.get_account(&alice.pubkey()).unwrap().lamports;

        // Cancel first
        CancelSubscription::new(&mut litesvm, &alice, plan_pda, subscription_pda)
            .execute()
            .assert_ok();

        // Advance clock past the expiration (plan has 1h period)
        move_clock_forward(&mut litesvm, hours(1));

        // Then revoke
        RevokeSubscription::new(&mut litesvm, &alice, subscription_pda)
            .execute()
            .assert_ok();

        // Account should be closed
        let account = litesvm.get_account(&subscription_pda);
        assert!(
            account.is_none() || account.as_ref().map(|a| a.lamports).unwrap_or(0) == 0,
            "Subscription PDA should be closed"
        );

        // Rent should be returned
        let balance_after = litesvm.get_account(&alice.pubkey()).unwrap().lamports;
        assert!(balance_after > balance_before - 10000);
    }

    #[test]
    fn revoke_subscription_with_future_expires_at_ts_rejected() {
        let (mut litesvm, alice, _merchant, mint, plan_pda, _, _subscription_pda) =
            setup_with_subscription();

        // Manually inject a subscription with expires_at_ts in the future
        let subscription_pda =
            CreateSubscription::new(&mut litesvm, plan_pda, alice.pubkey(), mint, current_ts())
                .expires_at_ts(current_ts() + days(1) as i64)
                .execute();

        let result = RevokeSubscription::new(&mut litesvm, &alice, subscription_pda).execute();
        result.assert_err(MultiDelegatorError::SubscriptionNotCancelled);

        // Account should still exist
        let account = litesvm.get_account(&subscription_pda);
        assert!(account.is_some());
    }

    #[test]
    fn test_revoke_fixed_version_agnostic() {
        use crate::state::header::VERSION_OFFSET;

        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, user, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, user, mint, delegatee)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        let mut account = litesvm.get_account(&delegation_pda).unwrap();
        account.data[VERSION_OFFSET] = 0;
        litesvm.set_account(delegation_pda, account).unwrap();

        RevokeDelegation::new(litesvm, user, mint, delegatee, nonce)
            .execute()
            .assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );
    }

    #[test]
    fn test_revoke_recurring_version_agnostic() {
        use crate::state::header::VERSION_OFFSET;

        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, user, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, user, mint, delegatee)
            .nonce(nonce)
            .recurring(100, days(1), current_ts(), current_ts() + days(2) as i64);
        res.assert_ok();

        let mut account = litesvm.get_account(&delegation_pda).unwrap();
        account.data[VERSION_OFFSET] = 0;
        litesvm.set_account(delegation_pda, account).unwrap();

        RevokeDelegation::new(litesvm, user, mint, delegatee, nonce)
            .execute()
            .assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );
    }

    #[test]
    fn test_revoke_subscription_version_mismatch() {
        use crate::state::header::VERSION_OFFSET;

        let (mut litesvm, alice, _merchant, _mint, plan_pda, _, subscription_pda) =
            setup_with_subscription();

        CancelSubscription::new(&mut litesvm, &alice, plan_pda, subscription_pda)
            .execute()
            .assert_ok();

        move_clock_forward(&mut litesvm, hours(1));

        let mut account = litesvm.get_account(&subscription_pda).unwrap();
        account.data[VERSION_OFFSET] = 0;
        litesvm.set_account(subscription_pda, account).unwrap();

        RevokeSubscription::new(&mut litesvm, &alice, subscription_pda)
            .execute()
            .assert_err(MultiDelegatorError::MigrationRequired);
    }

    #[test]
    fn sponsor_can_revoke_expired_fixed_delegation() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;
        let expiry_ts = current_ts() + hours(1) as i64;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(100, expiry_ts);
        res.assert_ok();

        let delegation_rent = litesvm.get_account(&delegation_pda).unwrap().lamports;

        move_clock_forward(litesvm, hours(2));

        let sponsor_balance_before = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;

        RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .signer(&sponsor)
            .execute()
            .assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let sponsor_balance_after = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;
        assert!(sponsor_balance_after >= sponsor_balance_before + delegation_rent - 10000);
    }

    #[test]
    fn sponsor_can_revoke_expired_recurring_delegation() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;
        let expiry_ts = current_ts() + days(2) as i64;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .recurring(100, days(1), current_ts(), expiry_ts);
        res.assert_ok();

        let delegation_rent = litesvm.get_account(&delegation_pda).unwrap().lamports;

        move_clock_forward(litesvm, days(3));

        let sponsor_balance_before = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;

        RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .signer(&sponsor)
            .execute()
            .assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let sponsor_balance_after = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;
        assert!(sponsor_balance_after >= sponsor_balance_before + delegation_rent - 10000);
    }

    #[test]
    fn sponsor_cannot_revoke_non_expired_fixed_delegation() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;
        let expiry_ts = current_ts() + hours(2) as i64;

        let (res, _) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(100, expiry_ts);
        res.assert_ok();

        RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .signer(&sponsor)
            .execute()
            .assert_err(MultiDelegatorError::Unauthorized);
    }

    #[test]
    fn sponsor_cannot_revoke_non_expired_recurring_delegation() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;
        let expiry_ts = current_ts() + days(2) as i64;

        let (res, _) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .recurring(100, days(1), current_ts(), expiry_ts);
        res.assert_ok();

        RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .signer(&sponsor)
            .execute()
            .assert_err(MultiDelegatorError::Unauthorized);
    }

    #[test]
    fn sponsor_cannot_revoke_no_expiry_delegation() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, _) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(100, 0);
        res.assert_ok();

        move_clock_forward(litesvm, days(365));

        RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .signer(&sponsor)
            .execute()
            .assert_err(MultiDelegatorError::Unauthorized);
    }

    #[test]
    fn delegator_can_revoke_sponsor_funded_before_expiry() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;
        let expiry_ts = current_ts() + hours(2) as i64;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(100, expiry_ts);
        res.assert_ok();

        let delegation_rent = litesvm.get_account(&delegation_pda).unwrap().lamports;
        let sponsor_balance_before = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;

        RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .receiver(sponsor.pubkey())
            .execute()
            .assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let sponsor_balance_after = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;
        assert!(sponsor_balance_after >= sponsor_balance_before + delegation_rent - 10000);
    }

    #[test]
    fn attacker_cannot_revoke_sponsor_funded_delegation() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);
        let attacker = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;
        let expiry_ts = current_ts() + hours(1) as i64;

        let (res, _) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(100, expiry_ts);
        res.assert_ok();

        move_clock_forward(litesvm, hours(2));

        // Attacker passes sponsor as receiver to try to close the account
        RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .signer(&attacker)
            .receiver(sponsor.pubkey())
            .execute()
            .assert_err(MultiDelegatorError::Unauthorized);
    }

    #[allow(clippy::result_large_err)]
    fn revoke_delegation_action_with_pda(
        litesvm: &mut litesvm::LiteSVM,
        signer: &solana_keypair::Keypair,
        delegation_pda: Pubkey,
        _multi_delegate_pda: Pubkey,
    ) -> litesvm::types::TransactionResult {
        use solana_instruction::{AccountMeta, Instruction};
        use solana_signer::Signer;

        use crate::{
            instructions::revoke_delegation,
            tests::{constants::PROGRAM_ID, utils::build_and_send_transaction},
        };

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(delegation_pda, false),
            ],
            data: vec![*revoke_delegation::DISCRIMINATOR],
        };

        build_and_send_transaction(litesvm, &[signer], &signer.pubkey(), &ix)
    }
}
