use std::time::{SystemTime, UNIX_EPOCH};

use litesvm::{types::TransactionResult, LiteSVM};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    solana_program::{
        clock::Clock,
        native_token::LAMPORTS_PER_SOL,
        program_pack::{IsInitialized, Pack},
    },
};
use spl_token_2022::{
    extension::{
        confidential_transfer::ConfidentialTransferMint, mint_close_authority::MintCloseAuthority,
        non_transferable::NonTransferable, pausable::PausableConfig,
        permanent_delegate::PermanentDelegate, transfer_fee::TransferFeeConfig,
        transfer_hook::TransferHook, BaseStateWithExtensionsMut, ExtensionType,
        StateWithExtensionsMut,
    },
    state::{Account as TokenAccount, AccountState, Mint as Mint2022},
};

use solana_instruction::AccountMeta;

use crate::{
    instructions::create_plan::{PlanData, MAX_DESTINATIONS, MAX_PULLERS},
    instructions::update_plan::UpdatePlanData,
    instructions::{
        close_multidelegate, create_fixed_delegation, create_plan, create_recurring_delegation,
        delete_plan, initialize_multidelegate, revoke_delegation, transfer_fixed_delegation,
        transfer_recurring_delegation, update_plan,
    },
    state::common::PlanStatus,
    tests::{
        constants::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
        cu_tracker::record_transaction,
        pda::{get_delegation_pda, get_multidelegate_pda, get_plan_pda},
    },
};

/// Converts number of minutes into seconds
pub fn minutes(mins: u64) -> u64 {
    mins * 60
}

/// Converts number of hours into seconds
pub fn hours(hours: u64) -> u64 {
    hours * minutes(60)
}

/// Converts number of days into seconds
pub fn days(days: u64) -> u64 {
    days * hours(24)
}

pub fn current_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn move_clock_forward(litesvm: &mut LiteSVM, seconds: u64) {
    let mut initial_clock = litesvm.get_sysvar::<Clock>();
    initial_clock.unix_timestamp += seconds as i64;
    litesvm.set_sysvar::<Clock>(&initial_clock);
}

pub fn get_ata_balance(litesvm: &LiteSVM, ata: &Pubkey) -> u64 {
    let account = fetch_account::<spl_token_2022::state::Account>(litesvm, ata);
    account.amount
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut litesvm = LiteSVM::new();

    let so_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/multi_delegator.so");
    litesvm
        .add_program_from_file(PROGRAM_ID.to_bytes(), so_path)
        .unwrap();

    let mut initial_clock = litesvm.get_sysvar::<Clock>();
    initial_clock.unix_timestamp = current_ts();
    litesvm.set_sysvar::<Clock>(&initial_clock);

    let default_payer = Keypair::new();
    litesvm
        .airdrop(&default_payer.pubkey(), LAMPORTS_PER_SOL * 100)
        .unwrap();

    (litesvm, default_payer)
}

fn pack_data<T: Pack>(state: T) -> Vec<u8> {
    let mut data = vec![0; T::LEN];
    T::pack(state, &mut data).unwrap();
    data
}

pub fn fetch_account<T: Pack + IsInitialized>(litesvm: &LiteSVM, pubkey: &Pubkey) -> T {
    let account = litesvm.get_account(pubkey).unwrap();
    T::unpack(account.data.as_ref()).unwrap()
}

#[allow(clippy::result_large_err)]
pub fn build_and_send_transaction(
    litesvm: &mut LiteSVM,
    signers: &[&Keypair],
    payer: &Pubkey,
    ix: &Instruction,
) -> TransactionResult {
    let tx = Transaction::new(
        signers,
        Message::new(std::slice::from_ref(ix), Some(payer)),
        litesvm.latest_blockhash(),
    );
    let result = litesvm.send_transaction(tx);
    litesvm.expire_blockhash();

    // Record CU consumption to global tracker
    record_transaction(&result, ix);

    result
}

pub fn init_wallet(litesvm: &mut LiteSVM, lamports: u64) -> Keypair {
    let wallet = Keypair::new();
    litesvm.airdrop(&wallet.pubkey(), lamports).unwrap();
    wallet
}

pub fn init_mint(
    litesvm: &mut LiteSVM,
    token_program: Pubkey,
    decimals: u8,
    supply: u64,
    authority: Option<Pubkey>,
    extensions: &[ExtensionType],
) -> Pubkey {
    let mint = Pubkey::new_unique();

    let space = if extensions.is_empty() {
        Mint2022::LEN
    } else {
        ExtensionType::try_calculate_account_len::<Mint2022>(extensions).unwrap()
    };
    let mut mint_data = vec![0u8; space];

    if extensions.is_empty() {
        let mint_state = Mint2022 {
            mint_authority: authority.into(),
            supply,
            decimals,
            is_initialized: true,
            freeze_authority: None.into(),
        };
        Mint2022::pack(mint_state, &mut mint_data).unwrap();
    } else {
        let mut state =
            StateWithExtensionsMut::<Mint2022>::unpack_uninitialized(&mut mint_data).unwrap();

        state.base.mint_authority = authority.into();
        state.base.supply = supply;
        state.base.decimals = decimals;
        state.base.is_initialized = true;
        state.base.freeze_authority = None.into();

        state.pack_base();
        state.init_account_type().unwrap();

        for ext in extensions {
            match ext {
                ExtensionType::ConfidentialTransferMint => {
                    state
                        .init_extension::<ConfidentialTransferMint>(true)
                        .unwrap();
                }
                ExtensionType::NonTransferable => {
                    state.init_extension::<NonTransferable>(true).unwrap();
                }
                ExtensionType::PermanentDelegate => {
                    state.init_extension::<PermanentDelegate>(true).unwrap();
                }
                ExtensionType::TransferFeeConfig => {
                    state.init_extension::<TransferFeeConfig>(true).unwrap();
                }
                ExtensionType::TransferHook => {
                    state.init_extension::<TransferHook>(true).unwrap();
                }
                ExtensionType::Pausable => {
                    state.init_extension::<PausableConfig>(true).unwrap();
                }
                ExtensionType::MintCloseAuthority => {
                    state.init_extension::<MintCloseAuthority>(true).unwrap();
                }
                _ => panic!("Unsupported extension type in test helper: {:?}", ext),
            }
        }
    }

    let lamports = litesvm.minimum_balance_for_rent_exemption(space);

    litesvm
        .set_account(
            mint,
            Account {
                lamports,
                data: mint_data,
                owner: token_program,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    mint
}

pub fn init_ata(litesvm: &mut LiteSVM, mint: Pubkey, owner: Pubkey, amount: u64) -> Pubkey {
    let token_program = litesvm.get_account(&mint).unwrap().owner;
    let ata = get_associated_token_address_with_program_id(&owner, &mint, &token_program);

    let ata_state = TokenAccount {
        mint,
        owner,
        amount,
        delegate: None.into(),
        state: AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    };

    let ata_data = pack_data(ata_state);
    let lamports = litesvm.minimum_balance_for_rent_exemption(TokenAccount::LEN);

    litesvm
        .set_account(
            ata,
            Account {
                lamports,
                data: ata_data,
                owner: token_program,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    ata
}

pub fn initialize_multidelegate_action(
    litesvm: &mut LiteSVM,
    payer: &Keypair,
    mint: Pubkey,
) -> (TransactionResult, Pubkey, u8) {
    let token_program = litesvm.get_account(&mint).unwrap().owner;
    let user_ata =
        get_associated_token_address_with_program_id(&payer.pubkey(), &mint, &token_program);
    let (multi_delegate_pda, bump) = get_multidelegate_pda(&payer.pubkey(), &mint);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: vec![*initialize_multidelegate::DISCRIMINATOR],
    };

    (
        build_and_send_transaction(litesvm, &[payer], &payer.pubkey(), &ix),
        multi_delegate_pda,
        bump,
    )
}

pub struct CreateDelegation<'a> {
    litesvm: &'a mut LiteSVM,
    delegator: &'a Keypair,
    payer: Option<&'a Keypair>,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
    custom_pda: Option<Pubkey>,
}

impl<'a> CreateDelegation<'a> {
    pub fn new(
        litesvm: &'a mut LiteSVM,
        delegator: &'a Keypair,
        mint: Pubkey,
        delegatee: Pubkey,
    ) -> Self {
        Self {
            litesvm,
            delegator,
            payer: None,
            mint,
            delegatee,
            nonce: 0,
            custom_pda: None,
        }
    }

    pub fn payer(mut self, payer: &'a Keypair) -> Self {
        self.payer = Some(payer);
        self
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    pub fn fixed(self, amount: u64, expiry_ts: i64) -> (TransactionResult, Pubkey) {
        let nonce_bytes = self.nonce.to_le_bytes().to_vec();
        self.execute(
            *create_fixed_delegation::DISCRIMINATOR,
            [
                nonce_bytes,
                amount.to_le_bytes().to_vec(),
                expiry_ts.to_le_bytes().to_vec(),
            ]
            .concat(),
        )
    }

    pub fn recurring(
        self,
        amount_per_period: u64,
        period_length_s: u64,
        start_ts: i64,
        expiry_ts: i64,
    ) -> (TransactionResult, Pubkey) {
        let nonce_bytes = self.nonce.to_le_bytes().to_vec();
        self.execute(
            *create_recurring_delegation::DISCRIMINATOR,
            [
                nonce_bytes,
                amount_per_period.to_le_bytes().to_vec(),
                period_length_s.to_le_bytes().to_vec(),
                start_ts.to_le_bytes().to_vec(),
                expiry_ts.to_le_bytes().to_vec(),
            ]
            .concat(),
        )
    }

    fn execute(self, discriminator: u8, data: Vec<u8>) -> (TransactionResult, Pubkey) {
        let (multi_delegate_pda, _) = get_multidelegate_pda(&self.delegator.pubkey(), &self.mint);
        let (derived_pda, _) = get_delegation_pda(
            &multi_delegate_pda,
            &self.delegator.pubkey(),
            &self.delegatee,
            self.nonce,
        );
        let delegation_pda = self.custom_pda.unwrap_or(derived_pda);

        let mut accounts = vec![
            AccountMeta::new(self.delegator.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(self.delegatee, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];

        let mut signers = vec![self.delegator];
        let mut fee_payer = self.delegator.pubkey();

        if let Some(p) = self.payer {
            accounts.push(AccountMeta::new(p.pubkey(), true));
            signers.push(p);
            fee_payer = p.pubkey();
        }

        // Instruction data now includes the bump at the end
        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: [vec![discriminator], data].concat(),
        };

        (
            build_and_send_transaction(self.litesvm, &signers, &fee_payer, &ix),
            delegation_pda,
        )
    }
}

pub struct TransferDelegation<'a> {
    litesvm: &'a mut LiteSVM,
    signer: &'a Keypair,
    delegator: Pubkey,
    mint: Pubkey,
    delegation_pda: Pubkey,
    amount: u64,
    receiver: Option<Pubkey>,
}

impl<'a> TransferDelegation<'a> {
    pub fn new(
        litesvm: &'a mut LiteSVM,
        signer: &'a Keypair,
        delegator: Pubkey,
        mint: Pubkey,
        delegation_pda: Pubkey,
    ) -> Self {
        Self {
            litesvm,
            signer,
            delegator,
            mint,
            delegation_pda,
            amount: 0,
            receiver: None,
        }
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn to(mut self, receiver: Pubkey) -> Self {
        self.receiver = Some(receiver);
        self
    }

    #[allow(clippy::result_large_err)]
    pub fn fixed(self) -> TransactionResult {
        self.execute(*transfer_fixed_delegation::DISCRIMINATOR)
    }

    #[allow(clippy::result_large_err)]
    pub fn recurring(self) -> TransactionResult {
        self.execute(*transfer_recurring_delegation::DISCRIMINATOR)
    }

    #[allow(clippy::result_large_err)]
    fn execute(self, discriminator: u8) -> TransactionResult {
        let token_program = self.litesvm.get_account(&self.mint).unwrap().owner;
        let (multi_delegate_pda, _) = get_multidelegate_pda(&self.delegator, &self.mint);
        let delegator_ata = get_associated_token_address_with_program_id(
            &self.delegator,
            &self.mint,
            &token_program,
        );

        // Default receiver is the signer's (delegatee's) ATA
        let receiver_ata = self.receiver.unwrap_or_else(|| {
            get_associated_token_address_with_program_id(
                &self.signer.pubkey(),
                &self.mint,
                &token_program,
            )
        });

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.delegation_pda, false),
                AccountMeta::new(multi_delegate_pda, false),
                AccountMeta::new(delegator_ata, false),
                AccountMeta::new(receiver_ata, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(self.signer.pubkey(), true),
            ],
            data: [
                vec![discriminator],
                self.amount.to_le_bytes().to_vec(),
                self.delegator.to_bytes().to_vec(),
                self.mint.to_bytes().to_vec(),
            ]
            .concat(),
        };

        build_and_send_transaction(self.litesvm, &[self.signer], &self.signer.pubkey(), &ix)
    }
}

pub struct RevokeDelegation<'a> {
    litesvm: &'a mut LiteSVM,
    delegator: &'a Keypair,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
    receiver: Option<Pubkey>,
    custom_pda: Option<Pubkey>,
}

impl<'a> RevokeDelegation<'a> {
    pub fn new(
        litesvm: &'a mut LiteSVM,
        delegator: &'a Keypair,
        mint: Pubkey,
        delegatee: Pubkey,
        nonce: u64,
    ) -> Self {
        Self {
            litesvm,
            delegator,
            mint,
            delegatee,
            nonce,
            receiver: None,
            custom_pda: None,
        }
    }

    pub fn receiver(mut self, receiver: Pubkey) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    #[allow(clippy::result_large_err)]
    pub fn execute(self) -> TransactionResult {
        let (multi_delegate_pda, _) = get_multidelegate_pda(&self.delegator.pubkey(), &self.mint);
        let (derived_pda, _) = get_delegation_pda(
            &multi_delegate_pda,
            &self.delegator.pubkey(),
            &self.delegatee,
            self.nonce,
        );
        let delegation_pda = self.custom_pda.unwrap_or(derived_pda);

        let mut accounts = vec![
            AccountMeta::new(self.delegator.pubkey(), true),
            AccountMeta::new(delegation_pda, false),
        ];

        if let Some(r) = self.receiver {
            accounts.push(AccountMeta::new(r, false));
        }

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: vec![*revoke_delegation::DISCRIMINATOR],
        };

        build_and_send_transaction(
            self.litesvm,
            &[self.delegator],
            &self.delegator.pubkey(),
            &ix,
        )
    }
}

pub struct CloseMultiDelegate<'a> {
    litesvm: &'a mut LiteSVM,
    user: &'a Keypair,
    mint: Pubkey,
    custom_pda: Option<Pubkey>,
}

impl<'a> CloseMultiDelegate<'a> {
    pub fn new(litesvm: &'a mut LiteSVM, user: &'a Keypair, mint: Pubkey) -> Self {
        Self {
            litesvm,
            user,
            mint,
            custom_pda: None,
        }
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    #[allow(clippy::result_large_err)]
    pub fn execute(self) -> TransactionResult {
        let (derived_pda, _) = get_multidelegate_pda(&self.user.pubkey(), &self.mint);
        let multi_delegate_pda = self.custom_pda.unwrap_or(derived_pda);

        let accounts = vec![
            AccountMeta::new(self.user.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
        ];

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: vec![*close_multidelegate::DISCRIMINATOR],
        };

        build_and_send_transaction(self.litesvm, &[self.user], &self.user.pubkey(), &ix)
    }
}

pub struct CreatePlan<'a> {
    litesvm: &'a mut LiteSVM,
    owner: &'a Keypair,
    data: PlanData,
    destinations_vec: Vec<Pubkey>,
    pullers_vec: Vec<Pubkey>,
    custom_pda: Option<Pubkey>,
}

impl<'a> CreatePlan<'a> {
    pub fn new(litesvm: &'a mut LiteSVM, owner: &'a Keypair, mint: Pubkey) -> Self {
        let zero_addr: pinocchio::Address = [0u8; 32].into();
        Self {
            litesvm,
            owner,
            data: PlanData {
                plan_id: 0,
                mint: mint.to_bytes().into(),
                amount: 0,
                period_hours: 0,
                end_ts: 0,
                destinations: [zero_addr; MAX_DESTINATIONS],
                pullers: [zero_addr; MAX_PULLERS],
                metadata_uri: [0u8; 128],
            },
            destinations_vec: vec![],
            pullers_vec: vec![],
            custom_pda: None,
        }
    }

    pub fn plan_id(mut self, plan_id: u64) -> Self {
        self.data.plan_id = plan_id;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.data.amount = amount;
        self
    }

    pub fn period_hours(mut self, period_hours: u64) -> Self {
        self.data.period_hours = period_hours;
        self
    }

    pub fn end_ts(mut self, end_ts: i64) -> Self {
        self.data.end_ts = end_ts;
        self
    }

    pub fn destinations(mut self, destinations: Vec<Pubkey>) -> Self {
        self.destinations_vec = destinations;
        self
    }

    pub fn pullers(mut self, pullers: Vec<Pubkey>) -> Self {
        self.pullers_vec = pullers;
        self
    }

    pub fn metadata_uri(mut self, uri: &str) -> Self {
        let bytes = uri.as_bytes();
        let len = bytes.len().min(128);
        self.data.metadata_uri[..len].copy_from_slice(&bytes[..len]);
        self
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    #[allow(clippy::result_large_err)]
    pub fn execute(mut self) -> (TransactionResult, Pubkey) {
        let (derived_pda, _) = get_plan_pda(&self.owner.pubkey(), self.data.plan_id);
        let plan_pda = self.custom_pda.unwrap_or(derived_pda);

        assert!(
            self.destinations_vec.len() <= MAX_DESTINATIONS,
            "max {MAX_DESTINATIONS} destinations"
        );
        let mut destinations = [[0u8; 32]; MAX_DESTINATIONS];
        for (i, d) in self.destinations_vec.iter().enumerate() {
            destinations[i] = d.to_bytes();
        }
        self.data.destinations = destinations.map(|d| d.into());

        assert!(
            self.pullers_vec.len() <= MAX_PULLERS,
            "max {MAX_PULLERS} pullers"
        );
        let mut pullers = [[0u8; 32]; MAX_PULLERS];
        for (i, p) in self.pullers_vec.iter().enumerate() {
            pullers[i] = p.to_bytes();
        }
        self.data.pullers = pullers.map(|p| p.into());

        let plan_data_bytes = unsafe {
            std::slice::from_raw_parts(&self.data as *const PlanData as *const u8, PlanData::LEN)
        };

        let mut data = vec![*create_plan::DISCRIMINATOR];
        data.extend_from_slice(plan_data_bytes);

        let mint_pubkey = Pubkey::new_from_array(self.data.mint.to_bytes());
        let token_program = self
            .litesvm
            .get_account(&mint_pubkey)
            .map(|a| a.owner)
            .unwrap_or(crate::tests::constants::TOKEN_PROGRAM_ID);

        let accounts = vec![
            AccountMeta::new(self.owner.pubkey(), true),
            AccountMeta::new(plan_pda, false),
            AccountMeta::new_readonly(mint_pubkey, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(token_program, false),
        ];

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data,
        };

        (
            build_and_send_transaction(self.litesvm, &[self.owner], &self.owner.pubkey(), &ix),
            plan_pda,
        )
    }
}

pub struct UpdatePlan<'a> {
    litesvm: &'a mut LiteSVM,
    owner: &'a Keypair,
    plan_pda: Pubkey,
    data: UpdatePlanData,
}

impl<'a> UpdatePlan<'a> {
    pub fn new(litesvm: &'a mut LiteSVM, owner: &'a Keypair, plan_pda: Pubkey) -> Self {
        Self {
            litesvm,
            owner,
            plan_pda,
            data: UpdatePlanData {
                status: PlanStatus::Active as u8,
                end_ts: 0,
                metadata_uri: [0u8; 128],
            },
        }
    }

    pub fn status(mut self, status: PlanStatus) -> Self {
        self.data.status = status as u8;
        self
    }

    pub fn status_raw(mut self, status: u8) -> Self {
        self.data.status = status;
        self
    }

    pub fn end_ts(mut self, end_ts: i64) -> Self {
        self.data.end_ts = end_ts;
        self
    }

    pub fn metadata_uri(mut self, uri: &str) -> Self {
        let bytes = uri.as_bytes();
        let len = bytes.len().min(128);
        self.data.metadata_uri = [0u8; 128];
        self.data.metadata_uri[..len].copy_from_slice(&bytes[..len]);
        self
    }

    #[allow(clippy::result_large_err)]
    pub fn execute(self) -> TransactionResult {
        let data_bytes = unsafe {
            std::slice::from_raw_parts(
                &self.data as *const UpdatePlanData as *const u8,
                UpdatePlanData::LEN,
            )
        };

        let mut data = vec![*update_plan::DISCRIMINATOR];
        data.extend_from_slice(data_bytes);

        let accounts = vec![
            AccountMeta::new(self.owner.pubkey(), true),
            AccountMeta::new(self.plan_pda, false),
        ];

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data,
        };

        build_and_send_transaction(self.litesvm, &[self.owner], &self.owner.pubkey(), &ix)
    }
}

pub struct DeletePlan<'a> {
    litesvm: &'a mut LiteSVM,
    owner: &'a Keypair,
    plan_pda: Pubkey,
}

impl<'a> DeletePlan<'a> {
    pub fn new(litesvm: &'a mut LiteSVM, owner: &'a Keypair, plan_pda: Pubkey) -> Self {
        Self {
            litesvm,
            owner,
            plan_pda,
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn execute(self) -> TransactionResult {
        let accounts = vec![
            AccountMeta::new(self.owner.pubkey(), true),
            AccountMeta::new(self.plan_pda, false),
        ];

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: vec![*delete_plan::DISCRIMINATOR],
        };

        build_and_send_transaction(self.litesvm, &[self.owner], &self.owner.pubkey(), &ix)
    }
}
