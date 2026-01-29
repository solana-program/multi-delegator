use core::mem::{size_of, transmute};
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, ProgramResult};
use shank::ShankType;

use crate::{
    create_delegation_account, init_header, CreateDelegationAccounts, DelegationKind,
    MultiDelegatorError, RecurringDelegation,
};

#[repr(C, packed)]
#[derive(Debug, ShankType)]
pub struct CreateRecurringDelegationData {
    pub nonce: u64,
    pub amount_per_period: u64,
    pub period_length_s: u64,
    pub expiry_s: u64,
}

impl CreateRecurringDelegationData {
    pub const LEN: usize =
        size_of::<u64>() + size_of::<u64>() + size_of::<u64>() + size_of::<u64>();

    fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            msg!(&format!(
                "Data.len() = {}. Expected = {}",
                data.len(),
                Self::LEN
            ));
            return Err(MultiDelegatorError::InvalidInstructionData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(data.as_ptr()) })
    }
}

pub const DISCRIMINATOR: &u8 = &2;

pub fn process((data, accounts): (&[u8], &[AccountInfo])) -> ProgramResult {
    let accounts = CreateDelegationAccounts::try_from(accounts)?;
    let call_data = CreateRecurringDelegationData::load(data)?;

    let bump = create_delegation_account(&accounts, call_data.nonce, RecurringDelegation::LEN)?;

    let binding = &mut accounts.delegation_account.try_borrow_mut_data()?;
    let delegation = RecurringDelegation::load_mut(binding)?;

    init_header(
        &mut delegation.header,
        DelegationKind::Recurring,
        bump,
        accounts.delegator.key(),
        accounts.delegatee.key(),
    );
    delegation.last_pull_ts = 0;
    delegation.period_length_s = call_data.period_length_s;
    delegation.expiry_s = call_data.expiry_s;
    delegation.amount_per_period = call_data.amount_per_period;
    delegation.amount_pulled_in_period = 0;

    Ok(())
}

#[cfg(test)]
mod tests {
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    use crate::{
        tests::{
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                create_recurring_delegation_action, init_ata, init_mint,
                initialize_multidelegate_action, setup,
            },
        },
        DelegationKind, RecurringDelegation,
    };

    #[test]
    fn create_recurring_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user;
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = 86400;
        let expiry_s: u64 = 2592000;
        let nonce: u64 = 0;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .unwrap();

        let delegatee = Pubkey::new_unique();

        let (res, delegation_pda) = create_recurring_delegation_action(
            litesvm,
            payer,
            mint,
            delegatee,
            nonce,
            amount_per_period,
            period_length_s,
            expiry_s,
        );
        res.unwrap();

        let account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&account.data).unwrap();

        let header = delegation.header;
        let del_amount_per_period = delegation.amount_per_period;
        let del_period_length_s = delegation.period_length_s;
        let del_expiry_s = delegation.expiry_s;
        let del_amount_pulled_in_period = delegation.amount_pulled_in_period;
        let del_last_pull_ts = delegation.last_pull_ts;
        assert_eq!(header.delegator, payer.pubkey().to_bytes());
        assert_eq!(header.delegatee, delegatee.to_bytes());
        assert_eq!(header.kind, DelegationKind::Recurring as u8);
        assert_eq!(del_amount_per_period, amount_per_period);
        assert_eq!(del_period_length_s, period_length_s);
        assert_eq!(del_expiry_s, expiry_s);
        assert_eq!(del_amount_pulled_in_period, 0);
        assert_eq!(del_last_pull_ts, 0);
    }
}
