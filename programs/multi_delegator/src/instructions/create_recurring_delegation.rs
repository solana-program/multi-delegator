use crate::{
    create_delegation_account, init_header, AccountDiscriminator, CreateDelegationAccounts,
    MultiDelegatorError, RecurringDelegation, DISCRIMINATOR_OFFSET,
};
use codama::CodamaType;
use core::mem::{size_of, transmute};
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio::{error::ProgramError, AccountView, ProgramResult};

use crate::constants::TIME_DRIFT_ALLOWED_SECS;

/// Maximum allowed period length for recurring delegations (365 days in seconds).
pub const MAX_DELEGATION_PERIOD_SECS: u64 = 31_536_000;

/// Instruction data payload for creating a recurring delegation.
#[repr(C, packed)]
#[derive(Debug, Clone, CodamaType)]
pub struct CreateRecurringDelegationData {
    /// Client-chosen nonce that disambiguates multiple delegations between the
    /// same delegator/delegatee pair.
    pub nonce: u64,
    /// Maximum token amount the delegatee may transfer per period.
    pub amount_per_period: u64,
    /// Length of each period in seconds (must be > 0 and <= [`MAX_DELEGATION_PERIOD_SECS`]).
    pub period_length_s: u64,
    /// Unix timestamp when the first period begins.
    pub start_ts: i64,
    /// Unix timestamp after which the delegation expires.
    pub expiry_ts: i64,
}

impl CreateRecurringDelegationData {
    /// Serialized size in bytes.
    pub const LEN: usize = size_of::<CreateRecurringDelegationData>();

    /// Zero-copy deserialize from raw instruction bytes.
    pub fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidInstructionData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(data.as_ptr()) })
    }

    /// Validates the instruction data against the current clock time.
    pub fn validate(&self, current_time: i64) -> Result<(), MultiDelegatorError> {
        if self.start_ts < current_time.saturating_sub(TIME_DRIFT_ALLOWED_SECS) {
            return Err(MultiDelegatorError::RecurringDelegationStartTimeInPast);
        }

        if self.period_length_s == 0 || self.period_length_s > MAX_DELEGATION_PERIOD_SECS {
            return Err(MultiDelegatorError::InvalidPeriodLength);
        }

        if self.start_ts >= self.expiry_ts {
            return Err(MultiDelegatorError::RecurringDelegationStartTimeGreaterThanExpiry);
        }

        if self.amount_per_period == 0 {
            return Err(MultiDelegatorError::RecurringDelegationAmountZero);
        }

        Ok(())
    }
}

/// Instruction discriminator byte for `CreateRecurringDelegation`.
pub const DISCRIMINATOR: &u8 = &2;

/// Creates a new [`RecurringDelegation`] PDA.
///
/// Validates the instruction data, creates the delegation account via CPI,
/// and initializes its header and period-tracking fields.
pub fn process(
    accounts: &[AccountView],
    call_data: &CreateRecurringDelegationData,
) -> ProgramResult {
    call_data.validate(Clock::get()?.unix_timestamp)?;

    let accounts = CreateDelegationAccounts::try_from(accounts)?;

    let bump = create_delegation_account(&accounts, call_data.nonce, RecurringDelegation::LEN)?;

    let binding = &mut accounts.delegation_account.try_borrow_mut()?;
    // Set discriminator before load_mut so validation passes on freshly created account
    binding[DISCRIMINATOR_OFFSET] = AccountDiscriminator::RecurringDelegation as u8;
    let delegation = RecurringDelegation::load_mut(binding)?;

    init_header(
        &mut delegation.header,
        AccountDiscriminator::RecurringDelegation,
        bump,
        accounts.delegator.address(),
        accounts.delegatee.address(),
        accounts.payer.address(),
    );
    delegation.current_period_start_ts = call_data.start_ts;
    delegation.period_length_s = call_data.period_length_s;
    delegation.expiry_ts = call_data.expiry_ts;
    delegation.amount_per_period = call_data.amount_per_period;
    delegation.amount_pulled_in_period = 0;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    use crate::tests::utils::current_ts;
    use crate::{
        tests::{
            asserts::TransactionResultExt,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                days, init_ata, init_mint, initialize_multidelegate_action, setup, CreateDelegation,
            },
        },
        AccountDiscriminator, MultiDelegatorError, RecurringDelegation,
    };

    #[test]
    fn create_recurring_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user;
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = 86400;
        let start_ts: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expiry_ts = start_ts + days(7) as i64;
        let nonce: u64 = 0;

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

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(amount_per_period, period_length_s, start_ts, expiry_ts);
        res.assert_ok();

        let account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&account.data).unwrap();

        let header = delegation.header;
        let del_amount_per_period = delegation.amount_per_period;
        let del_period_length_s = delegation.period_length_s;
        let del_expiry_s = delegation.expiry_ts;
        let del_amount_pulled_in_period = delegation.amount_pulled_in_period;
        let del_current_period_start_ts = delegation.current_period_start_ts;

        assert_eq!(header.delegator.to_bytes(), payer.pubkey().to_bytes());
        assert_eq!(header.delegatee.to_bytes(), delegatee.to_bytes());
        assert_eq!(header.payer.to_bytes(), payer.pubkey().to_bytes());
        assert_eq!(
            header.discriminator,
            AccountDiscriminator::RecurringDelegation as u8
        );
        assert_eq!(del_amount_per_period, amount_per_period);
        assert_eq!(del_period_length_s, period_length_s);
        assert_eq!(del_expiry_s, expiry_ts);
        assert_eq!(del_amount_pulled_in_period, 0);
        assert_eq!(del_current_period_start_ts, start_ts);
    }

    #[test]
    fn create_recurring_delegation_with_past_start_ts() {
        let (litesvm, user) = &mut setup();
        let payer = user;
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = 86400;
        let start_ts: i64 = i64::MIN;
        let expiry_ts = current_ts() + 100000000;
        let nonce: u64 = 0;

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

        let (res, _delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(amount_per_period, period_length_s, start_ts, expiry_ts);
        res.assert_err(MultiDelegatorError::RecurringDelegationStartTimeInPast);
    }

    #[test]
    fn create_recurring_delegation_with_zero_period() {
        let (litesvm, user) = &mut setup();
        let payer = user;
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = 0;
        let start_ts: i64 = current_ts() + 10000;
        let expiry_ts = current_ts() + 100000000;
        let nonce: u64 = 0;

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

        let (res, _delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(amount_per_period, period_length_s, start_ts, expiry_ts);
        res.assert_err(MultiDelegatorError::InvalidPeriodLength);
    }

    #[test]
    fn create_recurring_delegation_with_start_ts_greater_than_expiry_ts() {
        let (litesvm, user) = &mut setup();
        let payer = user;
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = 1;
        let start_ts: i64 = current_ts() + 100000000;
        let expiry_ts = current_ts() + 10000;
        let nonce: u64 = 0;

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

        let (res, _delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(amount_per_period, period_length_s, start_ts, expiry_ts);
        res.assert_err(MultiDelegatorError::RecurringDelegationStartTimeGreaterThanExpiry);
    }

    #[test]
    fn create_recurring_delegation_with_period_exceeding_max() {
        let (litesvm, user) = &mut setup();
        let payer = user;
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = 31_536_001;
        let start_ts: i64 = current_ts();
        let expiry_ts = current_ts() + days(365) as i64;
        let nonce: u64 = 0;

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

        let (res, _delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(amount_per_period, period_length_s, start_ts, expiry_ts);
        res.assert_err(MultiDelegatorError::InvalidPeriodLength);
    }
}
