use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::event_engine::verify_event_authority;

pub fn process(_program_id: &Address, accounts: &[AccountView]) -> ProgramResult {
    let [event_authority] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !event_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    verify_event_authority(event_authority)?;

    Ok(())
}
