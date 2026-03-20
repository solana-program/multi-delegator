/**
 * E2E tests for the Multi-Delegator devnet UI.
 *
 * Tests run serially and share on-chain state across the suite.
 * Required env vars in .env at repo root:
 *   PLAYRIGHT_WALLET  — base58-encoded 64-byte secret key for the test wallet
 *   PLAYWRIGHT_TOKEN_MINT — Token-2022 devnet mint the wallet holds an ATA for
 *
 * Optional:
 *   APP_URL — defaults to http://localhost:3000
 *
 * The test wallet must have devnet SOL for rent and a Token-2022 ATA for
 * PLAYWRIGHT_TOKEN_MINT (balance can be zero for delegation tests; non-zero
 * required for TransferFixed / TransferRecurring / TransferSubscription).
 */
import { expect, type Page, test } from '@playwright/test';

import { connectWallet, injectWallet } from './helpers/wallet';

// ─── Constants ────────────────────────────────────────────────────────────────

const TOKEN_MINT = process.env.PLAYWRIGHT_TOKEN_MINT ?? '';

// ─── Shared state (populated by earlier tests) ───────────────────────────────

let walletAddress = '';
let delegationPda = '';
let planPda = '';
let subscriptionPda = '';

// ─── Helpers ─────────────────────────────────────────────────────────────────

/**
 * Navigate to a panel via the sidebar.
 * `navLabel` is the sidebar button text; `headingName` is the h2 on the panel.
 * When they match, only one argument is needed.
 */
async function openPanel(page: Page, headingName: string, navLabel?: string): Promise<void> {
    await page.getByRole('button', { exact: true, name: navLabel ?? headingName }).click();
    await expect(page.getByRole('heading', { level: 2, name: headingName })).toBeVisible();
}

/** Click the nth Autofill button on the active panel. */
async function autofill(page: Page, nth = 0): Promise<void> {
    await page.getByRole('button', { name: 'Autofill' }).nth(nth).click();
}

/**
 * Clicks "Send Transaction" and waits for a new entry in Recent Transactions.
 * Snapshots the count before clicking to avoid TOCTOU races on fast devnet confirms.
 * Returns 'success' | 'failed'.
 */
async function sendAndWait(page: Page): Promise<'failed' | 'success'> {
    const heading = page.getByRole('heading', { name: /Recent Transactions/ });

    const beforeText = (await heading.textContent({ timeout: 500 }).catch(() => '')) ?? '';
    const beforeCount = parseInt(beforeText.match(/\d+/)?.[0] ?? '0');

    await page.getByRole('button', { name: 'Send Transaction' }).click();

    await expect(async () => {
        const text = (await heading.textContent()) ?? '';
        const count = parseInt(text.match(/\d+/)?.[0] ?? '0');
        expect(count).toBeGreaterThan(beforeCount);
    }).toPass({ intervals: [500, 1000, 2000], timeout: 45_000 });

    if (await page.getByText('Success', { exact: true }).last().isVisible()) return 'success';
    return 'failed';
}

// ─── Suite setup ─────────────────────────────────────────────────────────────

test.describe('Multi-Delegator UI', () => {
    test.describe.configure({ mode: 'serial' });

    let page: Page;

    test.beforeAll(async ({ browser }) => {
        const walletKey = process.env.PLAYRIGHT_WALLET;
        if (!walletKey) throw new Error('PLAYRIGHT_WALLET env var is not set');
        if (!TOKEN_MINT) throw new Error('PLAYWRIGHT_TOKEN_MINT env var is not set');

        page = await browser.newPage();
        await page.goto('/');
        walletAddress = await injectWallet(page, walletKey);
        await connectWallet(page);
    });

    test.afterAll(async () => {
        await page.close();
    });

    // ─── Init Multi-Delegate ─────────────────────────────────────────────────

    test('Init Multi-Delegate — succeeds and saves MultiDelegate PDA to QuickDefaults', async () => {
        await openPanel(page, 'Init Multi-Delegate');
        await page.getByRole('textbox', { name: 'Token Mint' }).fill(TOKEN_MINT);

        expect(await sendAndWait(page)).toBe('success');

        const defaultMultiDelegate = page.getByRole('combobox', { name: 'Default MultiDelegate' });
        await expect(defaultMultiDelegate).not.toHaveValue('');
        const saved = await defaultMultiDelegate.inputValue();
        expect(saved.length).toBeGreaterThanOrEqual(32);
        expect(saved.length).toBeLessThanOrEqual(44);

        await expect(page.getByRole('combobox', { name: 'Default Mint' })).toHaveValue(TOKEN_MINT);
        await expect(page.locator('text=1 saved').first()).toBeVisible();
    });

    // ─── Create Fixed Delegation ─────────────────────────────────────────────

    test('Create Fixed Delegation — succeeds and saves Delegation PDA to QuickDefaults', async () => {
        await openPanel(page, 'Create Fixed Delegation');
        await autofill(page, 0); // Mint
        // Delegatee = connected wallet (delegator = delegatee for test simplicity)
        await page.getByRole('textbox', { name: 'Delegatee' }).fill(walletAddress);
        await page.getByRole('spinbutton', { name: 'Nonce' }).fill('0');
        await page.getByRole('spinbutton', { name: 'Amount' }).fill('1000000');
        await page.getByRole('spinbutton', { name: 'Expiry Timestamp' }).fill('0');

        expect(await sendAndWait(page)).toBe('success');

        const defaultDelegation = page.getByRole('combobox', { name: 'Default Delegation' });
        await expect(defaultDelegation).not.toHaveValue('');
        delegationPda = await defaultDelegation.inputValue();
        expect(delegationPda.length).toBeGreaterThanOrEqual(32);
        expect(delegationPda.length).toBeLessThanOrEqual(44);

        await expect(page.getByRole('combobox', { name: 'Default Delegatee' })).toHaveValue(walletAddress);
    });

    // ─── Revoke Delegation ───────────────────────────────────────────────────

    test('Revoke Delegation — succeeds for the fixed delegation', async () => {
        await openPanel(page, 'Revoke Delegation');
        await autofill(page); // Delegation Account

        expect(await sendAndWait(page)).toBe('success');
    });

    // ─── Create Recurring Delegation ─────────────────────────────────────────

    test('Create Recurring Delegation — succeeds and overwrites Delegation PDA in QuickDefaults', async () => {
        await openPanel(page, 'Create Recurring Delegation');
        await autofill(page, 0); // Mint
        await page.getByRole('textbox', { name: 'Delegatee' }).fill(walletAddress);
        await page.getByRole('spinbutton', { name: 'Nonce' }).fill('0');
        await page.getByRole('spinbutton', { name: 'Amount Per Period' }).fill('500000');
        await page.getByRole('spinbutton', { name: 'Period Length (seconds)' }).fill('86400');
        await page.getByRole('spinbutton', { name: 'Expiry Timestamp' }).fill('0');
        await page.getByRole('spinbutton', { name: 'Start Timestamp' }).fill('0');

        expect(await sendAndWait(page)).toBe('success');

        const defaultDelegation = page.getByRole('combobox', { name: 'Default Delegation' });
        await expect(defaultDelegation).not.toHaveValue('');
        delegationPda = await defaultDelegation.inputValue();
    });

    // ─── Revoke Recurring Delegation ─────────────────────────────────────────

    test('Revoke Delegation — succeeds for the recurring delegation', async () => {
        await openPanel(page, 'Revoke Delegation');
        await autofill(page); // Delegation Account (now points to recurring)

        expect(await sendAndWait(page)).toBe('success');
    });

    // ─── Close Multi-Delegate ────────────────────────────────────────────────

    test('Close Multi-Delegate — succeeds once all delegations are revoked', async () => {
        await openPanel(page, 'Close Multi-Delegate');
        await autofill(page); // Mint

        expect(await sendAndWait(page)).toBe('success');
    });

    // ─── Create Plan ─────────────────────────────────────────────────────────
    // Re-init multi-delegate before subscribe test

    test('Init Multi-Delegate (second) — re-initialises for subscription tests', async () => {
        await openPanel(page, 'Init Multi-Delegate');
        await autofill(page); // Mint (from QuickDefaults)

        expect(await sendAndWait(page)).toBe('success');
    });

    test('Create Plan — succeeds and saves Plan PDA to QuickDefaults', async () => {
        await openPanel(page, 'Create Plan');
        await page.getByRole('spinbutton', { name: 'Plan ID' }).fill('0');
        await autofill(page); // Mint
        await page.getByRole('spinbutton', { name: 'Amount' }).fill('1000000');
        await page.getByRole('spinbutton', { name: 'Period Hours' }).fill('24');
        await page.getByRole('spinbutton', { name: 'End Timestamp' }).fill('0');

        expect(await sendAndWait(page)).toBe('success');

        const defaultPlan = page.getByRole('combobox', { name: 'Default Plan' });
        await expect(defaultPlan).not.toHaveValue('');
        planPda = await defaultPlan.inputValue();
        expect(planPda.length).toBeGreaterThanOrEqual(32);
        expect(planPda.length).toBeLessThanOrEqual(44);
    });

    // ─── Update Plan ─────────────────────────────────────────────────────────

    test('Update Plan — succeeds setting metadata URI', async () => {
        await openPanel(page, 'Update Plan');
        await autofill(page); // Plan PDA
        await page.getByRole('textbox', { name: 'Metadata URI' }).fill('https://multidelegator.test/plan.json');

        expect(await sendAndWait(page)).toBe('success');
    });

    // ─── Subscribe ───────────────────────────────────────────────────────────

    test('Subscribe — succeeds and saves Subscription PDA to QuickDefaults', async () => {
        await openPanel(page, 'Subscribe');
        // Merchant = connected wallet (subscribed to own plan for test purposes)
        await page.getByRole('textbox', { name: 'Merchant' }).fill(walletAddress);
        await page.getByRole('spinbutton', { name: 'Plan ID' }).fill('0');
        await autofill(page); // Token Mint

        expect(await sendAndWait(page)).toBe('success');

        const defaultSubscription = page.getByRole('combobox', { name: 'Default Subscription' });
        await expect(defaultSubscription).not.toHaveValue('');
        subscriptionPda = await defaultSubscription.inputValue();
        expect(subscriptionPda.length).toBeGreaterThanOrEqual(32);
        expect(subscriptionPda.length).toBeLessThanOrEqual(44);
    });

    // ─── Cancel Subscription ─────────────────────────────────────────────────

    test('Cancel Subscription — succeeds', async () => {
        await openPanel(page, 'Cancel Subscription');
        await autofill(page, 0); // Plan PDA
        await autofill(page, 1); // Subscription PDA

        expect(await sendAndWait(page)).toBe('success');
    });

    // ─── Update Plan to Sunset ───────────────────────────────────────────────

    test('Update Plan — succeeds setting status to Sunset', async () => {
        await openPanel(page, 'Update Plan');
        await autofill(page); // Plan PDA
        await page.getByRole('combobox', { name: 'Status' }).selectOption('Sunset');
        await page.getByRole('spinbutton', { name: 'End Timestamp' }).fill('1');

        expect(await sendAndWait(page)).toBe('success');
    });

    // ─── UI components ───────────────────────────────────────────────────────

    test.describe('UI components', () => {
        test('RPC badge opens dropdown with network presets and custom URL input', async () => {
            await page.getByRole('button', { name: /Devnet/ }).click();
            await expect(page.getByRole('button', { name: /Mainnet/i })).toBeVisible();
            await expect(page.getByRole('button', { name: /Testnet/i })).toBeVisible();
            await expect(page.getByRole('button', { name: /Localhost/i })).toBeVisible();
            await expect(page.getByRole('textbox', { name: /my-rpc/i })).toBeVisible();
            await page.keyboard.press('Escape');
        });

        test('Program badge opens with editable program ID', async () => {
            await page.getByRole('button', { name: /Default Program/ }).click();
            await expect(page.getByRole('button', { name: 'Set Program ID' })).toBeVisible();
            await expect(page.getByRole('button', { name: 'Use Default' })).toBeVisible();
            await page.keyboard.press('Escape');
        });

        test('QuickDefaults Clear Saved removes all saved values', async () => {
            await expect(page.getByRole('combobox', { name: 'Default Mint' })).not.toHaveValue('');

            await page.getByRole('button', { name: 'Clear Saved' }).click();

            await expect(page.getByRole('combobox', { name: 'Default Mint' })).toHaveValue('');
            await expect(page.getByRole('combobox', { name: 'Default Plan' })).toHaveValue('');
            await expect(page.getByRole('combobox', { name: 'Default Subscription' })).toHaveValue('');
            await expect(page.locator('text=0 saved').first()).toBeVisible();
        });

        test('RecentTransactions shows all transactions with View Explorer links', async () => {
            const heading = page.getByRole('heading', { name: /Recent Transactions \(\d+\)/ });
            await expect(heading).toBeVisible();

            const count = parseInt((await heading.textContent())!.match(/\d+/)![0]);
            // Init×2, CreateFixed, Revoke×2, CreateRecurring, CloseMultiDelegate,
            // CreatePlan, UpdatePlan×2, Subscribe, CancelSubscription
            expect(count).toBeGreaterThanOrEqual(10);

            await expect(page.getByRole('button', { name: 'View Explorer' }).first()).toBeVisible();
        });
    });
});
