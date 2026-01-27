use core::mem::{size_of, transmute};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use pinocchio_system::instructions::CreateAccount;
use shank::ShankType;

use crate::{
    AccountCheck, TermsState, OneTimeTerms, MultiDelegatorError, SignerAccount,
    SystemAccount, DELEGATE_BASE_SEED,
};

pub struct AddDelegateAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub multi_delegate: &'a AccountInfo,
    pub delegate_account: &'a AccountInfo,
    pub delegate: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for AddDelegateAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, multi_delegate, delegate_account, delegate, system_program, ..] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(owner)?;
        SystemAccount::check(system_program)?;
        // Check to make sure delegate is non existant

        Ok(Self {
            owner,
            multi_delegate,
            delegate_account,
            delegate,
            system_program,
        })
    }
}

/// Packed removes 7 bytes of padding that the compiler would add between `kind` (u8) and
/// `amount` (u64) for 8-byte alignment. Without packed: 24 bytes. With packed: 17 bytes.
/// Required here because instruction data from the client is serialized without padding,
/// so transmute needs the struct layout to match exactly.
#[repr(C, packed)]
#[derive(Debug, ShankType)]
pub struct AddDelegateInstructionData {
    pub kind: u8,
    pub amount: u64,
    pub expiry_s: u64,
}

impl AddDelegateInstructionData {
    pub const LEN: usize = size_of::<u8>() + size_of::<u64>() + size_of::<u64>();

    fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        // TODO better range checks
        if data.len() != AddDelegateInstructionData::LEN {
            msg!(&format!(
                "Data.len() = {}. Expected =  {}",
                data.len(),
                AddDelegateInstructionData::LEN
            ));
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(data.as_ptr()) })
    }
}

pub const DISCRIMINATOR: &u8 = &1;

pub fn process((data, accounts): (&[u8], &[AccountInfo])) -> ProgramResult {
    msg!("at start of add_delegate");
    let accounts = AddDelegateAccounts::try_from(accounts)?;
    let call_data = AddDelegateInstructionData::load(data)?;
    msg!("Accounts and calldata parsed correctly");
    msg!(&format!("Calldata: {:?}", call_data));

    let (expected_pda, bump) = OneTimeTerms::find_pda(
        accounts.multi_delegate.key(),
        accounts.delegate.key(),
        accounts.owner.key(),
        call_data.kind.try_into()?,
    );

    if expected_pda != *accounts.delegate_account.key() {
        return Err(MultiDelegatorError::InvalidDelegatePda.into());
    }

    msg!("Correct PDA provided");

    let lamports = Rent::get()?.minimum_balance(OneTimeTerms::LEN);
    let bump_bytes = [bump];
    let kind_bytes = [call_data.kind];
    let seeds = [
        Seed::from(DELEGATE_BASE_SEED),
        Seed::from(accounts.multi_delegate.key().as_ref()),
        Seed::from(accounts.delegate.key().as_ref()),
        Seed::from(accounts.owner.key().as_ref()),
        Seed::from(&kind_bytes),
        Seed::from(&bump_bytes),
    ];
    let signer = [Signer::from(&seeds)];

    msg!("About to create and initialize the account");

    CreateAccount {
        from: accounts.owner,
        to: accounts.delegate_account,
        lamports,
        space: OneTimeTerms::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&signer)?;

    msg!("Created the account, now time to set the data");

    let binding = &mut accounts.delegate_account.try_borrow_mut_data()?;
    let delegation_state = OneTimeTerms::load_mut(binding)?;

    delegation_state.kind = call_data.kind.try_into()?;
    delegation_state.delegator = *accounts.owner.key();
    delegation_state.status = TermsState::Active;
    delegation_state.max_amount = call_data.amount;
    delegation_state.remaining_amount = call_data.amount;
    delegation_state.expiry_s = call_data.expiry_s;

    msg!(&format!("Delegation state: {:?}", delegation_state));

    msg!("All data set");

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
                create_simple_delegate_action, init_ata, init_mint,
                initialize_multidelegate_action, setup,
            },
        },
        TermsKind, OneTimeTerms,
    };

    #[test]
    fn create_fixed_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user; // Simplification for test
        let amount: u64 = 100_000_000;
        let expiry_s: u64 = 1000;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        // Initialize MultiDelegate first
        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .unwrap();

        let delegate = Pubkey::new_unique();
        let kind = TermsKind::OneTime;

        let (res, delegate_pda) =
            create_simple_delegate_action(litesvm, payer, mint, delegate, amount, expiry_s, kind);
        res.unwrap();

        let account = litesvm.get_account(&delegate_pda).unwrap();

        let delegate = OneTimeTerms::load(&account.data).unwrap();

        assert_eq!(delegate.delegator, payer.pubkey().to_bytes());
        assert_eq!(delegate.kind, TermsKind::OneTime);
        assert_eq!(delegate.remaining_amount, amount);
        assert_eq!(delegate.max_amount, amount);
        assert_eq!(delegate.expiry_s, expiry_s);
    }
}
