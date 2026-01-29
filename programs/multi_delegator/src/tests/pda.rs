use solana_pubkey::Pubkey;

use crate::{tests::constants::PROGRAM_ID, MultiDelegate, DELEGATE_BASE_SEED};

pub fn get_multidelegate_pda(user: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MultiDelegate::SEED, user.as_ref(), token_mint.as_ref()],
        &PROGRAM_ID,
    )
}

pub fn get_delegation_pda(
    multidelegate: &Pubkey,
    delegator: &Pubkey,
    delegatee: &Pubkey,
    nonce: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            DELEGATE_BASE_SEED,
            multidelegate.as_ref(),
            delegator.as_ref(),
            delegatee.as_ref(),
            &nonce.to_le_bytes(),
        ],
        &PROGRAM_ID,
    )
}
