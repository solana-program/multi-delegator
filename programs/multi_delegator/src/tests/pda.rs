use solana_pubkey::Pubkey;

use crate::{tests::constants::PROGRAM_ID, MultiDelegate, TermsKind, DELEGATE_BASE_SEED};

pub fn get_multidelegate_pda(user: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MultiDelegate::SEED, user.as_ref(), token_mint.as_ref()],
        &PROGRAM_ID,
    )
}

pub fn get_delegate_pda(
    multidelegate: &Pubkey,
    delegate: &Pubkey,
    payer: &Pubkey,
    kind: TermsKind,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            DELEGATE_BASE_SEED,
            multidelegate.as_ref(),
            delegate.as_ref(),
            payer.as_ref(),
            &[kind as u8],
        ],
        &PROGRAM_ID,
    )
}
