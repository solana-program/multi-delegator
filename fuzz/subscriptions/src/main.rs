use std::rc::Rc;

use crucible_fuzzer::*;
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use subscriptions::accounts::{EventAuthority, Plan, SubscriptionAuthority, SubscriptionDelegation};
use subscriptions::instructions::{
    CancelSubscriptionBuilder, CreatePlanBuilder, InitSubscriptionAuthorityBuilder, SubscribeBuilder,
};
use subscriptions::types::{PlanData, PlanTerms, SubscribeData};
use subscriptions::SUBSCRIPTIONS_ID;

const TOKEN_PROGRAM_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ATA_PROGRAM_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

const GENESIS_TS: i64 = 1_753_000_000;
const SLOTS_PER_SECOND: i64 = 2;
const INITIAL_LAMPORTS: u64 = 10_000_000_000;
const INITIAL_TOKENS: u64 = 1_000_000_000;
const PLAN_ID: u64 = 1;
const PLAN_AMOUNT: u64 = 1_000_000;
const PLAN_PERIOD_HOURS: u64 = 24;
const SUBSCRIBER_COUNT: usize = 3;

fn ata_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()], &ATA_PROGRAM_ID).0
}

fn plan_pda_address(owner: &Pubkey, plan_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Plan::PREFIX, owner.as_ref(), &plan_id.to_le_bytes()], &SUBSCRIPTIONS_ID)
}

#[derive(Clone)]
struct SubscriptionsFixture {
    ctx: TestContext,
    mint: Pubkey,
    merchant: Rc<Keypair>,
    merchant_ata: Pubkey,
    subscribers: Vec<Rc<Keypair>>,
    plan_pda: Pubkey,
    plan_bump: u8,
    now_ts: i64,
}

impl SubscriptionsFixture {
    fn set_time(&mut self, ts: i64) {
        self.now_ts = ts;
        set_clock(&mut self.ctx, ts);
    }

    fn read_plan(&self) -> Option<Plan> {
        let account = self.ctx.get_account(&self.plan_pda).ok()?;
        Plan::from_bytes(&account.data).ok()
    }

    fn read_authority(&self, subscriber: &Pubkey) -> Option<SubscriptionAuthority> {
        let (authority_pda, _) = SubscriptionAuthority::find_pda(subscriber, &self.mint);
        let account = self.ctx.get_account(&authority_pda).ok()?;
        SubscriptionAuthority::from_bytes(&account.data).ok()
    }
}

fn set_clock(ctx: &mut TestContext, ts: i64) {
    let slot = ((ts - GENESIS_TS).max(0) * SLOTS_PER_SECOND) as u64 + 1;
    ctx.warp_to_slot(slot);
    ctx.set_sysvar(&Clock {
        slot,
        epoch_start_timestamp: GENESIS_TS,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: ts,
    });
}

fn create_funded_wallet(ctx: &mut TestContext) -> Rc<Keypair> {
    let wallet = Rc::new(Keypair::new());
    ctx.create_account()
        .pubkey(wallet.pubkey())
        .lamports(INITIAL_LAMPORTS)
        .owner(anchor_lang::system_program::ID)
        .create()
        .expect("create wallet");
    wallet
}

fn create_ata(ctx: &mut TestContext, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
    let ata = ata_address(owner, mint);
    ctx.create_token_account()
        .pubkey(ata)
        .mint(*mint)
        .token_owner(*owner)
        .amount(amount)
        .create()
        .expect("create token account");
    ata
}

#[fuzz_fixture]
impl SubscriptionsFixture {
    pub fn setup() -> Self {
        let mut ctx = TestContext::new();
        ctx.add_program(&SUBSCRIPTIONS_ID, "../../target/deploy/subscriptions_program.so")
            .expect("load subscriptions program");
        set_clock(&mut ctx, GENESIS_TS);
        // litesvm 0.9 (pinned by crucible) serves feature-gated rent (exemption_threshold 1.0)
        // through the Rent::get() syscall but validates rent exemption against this sysvar
        // account, which defaults to 2.0. Align the sysvar so program-funded PDAs pass.
        ctx.set_sysvar(&anchor_lang::prelude::Rent { exemption_threshold: 1.0, ..Default::default() });

        let mint_authority = Keypair::new();
        let mint = Pubkey::new_unique();
        ctx.create_mint()
            .pubkey(mint)
            .mint_authority(mint_authority.pubkey())
            .decimals(6)
            .create()
            .expect("create mint");

        let merchant = create_funded_wallet(&mut ctx);
        let merchant_ata = create_ata(&mut ctx, &merchant.pubkey(), &mint, 0);

        let mut subscribers = Vec::with_capacity(SUBSCRIBER_COUNT);
        for _ in 0..SUBSCRIBER_COUNT {
            let subscriber = create_funded_wallet(&mut ctx);
            let user_ata = create_ata(&mut ctx, &subscriber.pubkey(), &mint, INITIAL_TOKENS);
            let (authority_pda, _) = SubscriptionAuthority::find_pda(&subscriber.pubkey(), &mint);
            let ix = InitSubscriptionAuthorityBuilder::new()
                .owner(subscriber.pubkey())
                .subscription_authority(authority_pda)
                .token_mint(mint)
                .user_ata(user_ata)
                .token_program(TOKEN_PROGRAM_ID)
                .instruction();
            ctx.raw_call(ix).signers(&[&subscriber]).send().expect("send init authority").unwrap();
            subscribers.push(subscriber);
        }

        let (plan_pda, plan_bump) = plan_pda_address(&merchant.pubkey(), PLAN_ID);
        let mut destinations = [Pubkey::default(); 4];
        destinations[0] = merchant_ata;
        let mut pullers = [Pubkey::default(); 4];
        pullers[0] = merchant.pubkey();
        let ix = CreatePlanBuilder::new()
            .merchant(merchant.pubkey())
            .plan_pda(plan_pda)
            .token_mint(mint)
            .plan_data(PlanData {
                plan_id: PLAN_ID,
                mint,
                terms: PlanTerms { amount: PLAN_AMOUNT, period_hours: PLAN_PERIOD_HOURS, created_at: 0 },
                end_ts: 0,
                destinations,
                pullers,
                metadata_uri: [0u8; 128],
            })
            .instruction();
        ctx.raw_call(ix).signers(&[&merchant]).send().expect("send create plan").unwrap();

        Self { ctx, mint, merchant, merchant_ata, subscribers, plan_pda, plan_bump, now_ts: GENESIS_TS }
    }

    pub fn action_advance_time(&mut self, #[range(1..73)] hours: i64) {
        self.set_time(self.now_ts + hours * 3600);
    }

    pub fn action_subscribe(&mut self, #[range(0..3)] subscriber_idx: usize) -> bool {
        let subscriber = self.subscribers[subscriber_idx].clone();
        let Some(plan) = self.read_plan() else { return false };
        let Some(authority) = self.read_authority(&subscriber.pubkey()) else { return false };
        let (authority_pda, _) = SubscriptionAuthority::find_pda(&subscriber.pubkey(), &self.mint);
        let (subscription_pda, _) = SubscriptionDelegation::find_pda(&self.plan_pda, &subscriber.pubkey());
        let ix = SubscribeBuilder::new()
            .subscriber(subscriber.pubkey())
            .merchant(self.merchant.pubkey())
            .plan_pda(self.plan_pda)
            .subscription_pda(subscription_pda)
            .subscription_authority_pda(authority_pda)
            .event_authority(EventAuthority::find_pda().0)
            .subscribe_data(SubscribeData {
                plan_id: PLAN_ID,
                plan_bump: self.plan_bump,
                expected_mint: plan.data.mint,
                expected_amount: plan.data.terms.amount,
                expected_period_hours: plan.data.terms.period_hours,
                expected_created_at: plan.data.terms.created_at,
                expected_subscription_authority_init_id: authority.init_id,
            })
            .instruction();
        self.ctx.raw_call(ix).signers(&[&subscriber]).send().map(|outcome| outcome.is_success()).unwrap_or(false)
    }

    pub fn action_cancel_subscription(&mut self, #[range(0..3)] subscriber_idx: usize) -> bool {
        let subscriber = self.subscribers[subscriber_idx].clone();
        let (subscription_pda, _) = SubscriptionDelegation::find_pda(&self.plan_pda, &subscriber.pubkey());
        let ix = CancelSubscriptionBuilder::new()
            .subscriber(subscriber.pubkey())
            .plan_pda(self.plan_pda)
            .subscription_pda(subscription_pda)
            .event_authority(EventAuthority::find_pda().0)
            .instruction();
        self.ctx.raw_call(ix).signers(&[&subscriber]).send().map(|outcome| outcome.is_success()).unwrap_or(false)
    }
}

#[invariant_test]
fn invariant_subscriptions(fixture: &mut SubscriptionsFixture) {
    for subscriber in &fixture.subscribers {
        let (subscription_pda, _) = SubscriptionDelegation::find_pda(&fixture.plan_pda, &subscriber.pubkey());
        if !fixture.ctx.account_has_data(&subscription_pda, 1) {
            continue;
        }
        let account = fixture.ctx.get_account(&subscription_pda).expect("subscription account");
        let subscription = SubscriptionDelegation::from_bytes(&account.data);
        fuzz_assert!(subscription.is_ok(), "subscription account must stay decodable");
        let Ok(subscription) = subscription else { continue };
        fuzz_assert_le!(
            subscription.amount_pulled_in_period,
            subscription.terms.amount,
            "amount pulled in period exceeds authorized plan amount"
        );
    }
}
