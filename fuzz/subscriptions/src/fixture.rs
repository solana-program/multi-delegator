use std::rc::Rc;

use crucible_fuzzer::*;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use subscriptions::accounts::{EventAuthority, Plan, SubscriptionAuthority, SubscriptionDelegation};
use subscriptions::instructions::{
    CancelSubscriptionBuilder, CancelSubscriptionNowBuilder, CreatePlanBuilder, InitSubscriptionAuthorityBuilder,
    ResumeSubscriptionBuilder, SubscribeBuilder, TransferSubscriptionBuilder,
};
use subscriptions::types::{CancelSubscriptionNowData, PlanData, PlanTerms, ResumeData, SubscribeData, TransferData};
use subscriptions::SUBSCRIPTIONS_ID;

use crate::constants::{
    GENESIS_TS, INITIAL_TOKENS, MINT_DECIMALS, PLAN_AMOUNT, PLAN_ID, PLAN_PERIOD_HOURS, SUBSCRIBER_COUNT,
};
use crate::helpers::{ata_address, create_ata, create_funded_wallet, plan_pda_address, set_clock};

#[derive(Clone)]
pub struct SubscriptionsFixture {
    pub ctx: TestContext,
    pub mint: Pubkey,
    pub merchant: Rc<Keypair>,
    pub merchant_ata: Pubkey,
    pub subscribers: Vec<Rc<Keypair>>,
    pub plan_pda: Pubkey,
    pub plan_bump: u8,
    pub now_ts: i64,
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

    pub fn subscription_pda(&self, subscriber: &Pubkey) -> Pubkey {
        SubscriptionDelegation::find_pda(&self.plan_pda, subscriber).0
    }

    fn read_subscription(&self, subscriber: &Pubkey) -> Option<SubscriptionDelegation> {
        let account = self.ctx.get_account(&self.subscription_pda(subscriber)).ok()?;
        SubscriptionDelegation::from_bytes(&account.data).ok()
    }
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
        ctx.set_sysvar(&solana_rent::Rent { exemption_threshold: 1.0, ..Default::default() });

        let mint_authority = Keypair::new();
        let mint = Pubkey::new_unique();
        ctx.create_mint()
            .pubkey(mint)
            .mint_authority(mint_authority.pubkey())
            .decimals(MINT_DECIMALS)
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
                .token_program(spl_token_interface::ID)
                .instruction();
            ctx.raw_call(ix).signers(&[&subscriber]).send().expect("send init authority").unwrap();
            subscribers.push(subscriber);
        }

        let (plan_pda, plan_bump) = plan_pda_address(&merchant.pubkey(), PLAN_ID);
        let mut destinations = [Pubkey::default(); 4];
        destinations[0] = merchant.pubkey();
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
        let ix = SubscribeBuilder::new()
            .subscriber(subscriber.pubkey())
            .merchant(self.merchant.pubkey())
            .plan_pda(self.plan_pda)
            .subscription_pda(self.subscription_pda(&subscriber.pubkey()))
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

    pub fn action_pull_payment(
        &mut self,
        #[range(0..3)] subscriber_idx: usize,
        #[range(0..2_000_001)] amount: u64,
    ) -> bool {
        let subscriber = self.subscribers[subscriber_idx].clone();
        if !self.ctx.account_has_data(&self.subscription_pda(&subscriber.pubkey()), 1) {
            return false;
        }
        let (authority_pda, _) = SubscriptionAuthority::find_pda(&subscriber.pubkey(), &self.mint);
        let ix = TransferSubscriptionBuilder::new()
            .subscription_pda(self.subscription_pda(&subscriber.pubkey()))
            .plan_pda(self.plan_pda)
            .subscription_authority(authority_pda)
            .delegator_ata(ata_address(&subscriber.pubkey(), &self.mint))
            .receiver_ata(self.merchant_ata)
            .caller(self.merchant.pubkey())
            .token_mint(self.mint)
            .token_program(spl_token_interface::ID)
            .event_authority(EventAuthority::find_pda().0)
            .transfer_data(TransferData { amount, delegator: subscriber.pubkey(), mint: self.mint })
            .instruction();
        self.ctx
            .raw_call(ix)
            .signers(&[&self.merchant.clone()])
            .send()
            .map(|outcome| outcome.is_success())
            .unwrap_or(false)
    }

    pub fn action_cancel_subscription(&mut self, #[range(0..3)] subscriber_idx: usize) -> bool {
        let subscriber = self.subscribers[subscriber_idx].clone();
        let ix = CancelSubscriptionBuilder::new()
            .subscriber(subscriber.pubkey())
            .plan_pda(self.plan_pda)
            .subscription_pda(self.subscription_pda(&subscriber.pubkey()))
            .event_authority(EventAuthority::find_pda().0)
            .instruction();
        self.ctx.raw_call(ix).signers(&[&subscriber]).send().map(|outcome| outcome.is_success()).unwrap_or(false)
    }

    pub fn action_resume_subscription(&mut self, #[range(0..3)] subscriber_idx: usize) -> bool {
        let subscriber = self.subscribers[subscriber_idx].clone();
        let Some(subscription) = self.read_subscription(&subscriber.pubkey()) else { return false };
        let (authority_pda, _) = SubscriptionAuthority::find_pda(&subscriber.pubkey(), &self.mint);
        let ix = ResumeSubscriptionBuilder::new()
            .subscriber(subscriber.pubkey())
            .plan_pda(self.plan_pda)
            .subscription_pda(self.subscription_pda(&subscriber.pubkey()))
            .subscription_authority(authority_pda)
            .event_authority(EventAuthority::find_pda().0)
            .resume_data(ResumeData { expected_expires_at_ts: subscription.expires_at_ts })
            .instruction();
        self.ctx.raw_call(ix).signers(&[&subscriber]).send().map(|outcome| outcome.is_success()).unwrap_or(false)
    }

    pub fn action_cancel_subscription_now(&mut self, #[range(0..3)] subscriber_idx: usize) -> bool {
        let subscriber = self.subscribers[subscriber_idx].clone();
        let Some(subscription) = self.read_subscription(&subscriber.pubkey()) else { return false };
        let ix = CancelSubscriptionNowBuilder::new()
            .subscriber(subscriber.pubkey())
            .merchant(self.merchant.pubkey())
            .plan_pda(self.plan_pda)
            .subscription_pda(self.subscription_pda(&subscriber.pubkey()))
            .event_authority(EventAuthority::find_pda().0)
            .cancel_subscription_now_data(CancelSubscriptionNowData {
                expected_current_period_start_ts: subscription.current_period_start_ts,
            })
            .instruction();
        self.ctx
            .raw_call(ix)
            .signers(&[&subscriber, &self.merchant.clone()])
            .send()
            .map(|outcome| outcome.is_success())
            .unwrap_or(false)
    }
}
