import { AccountRole, generateKeyPairSigner, type Address } from '@solana/kit';
import { describe, expect, test } from 'vitest';

import { getCreatePlanInstruction } from '../src/index.ts';

const ZERO_ADDRESS = '11111111111111111111111111111111' as Address;

async function planInputs() {
    const merchant = await generateKeyPairSigner();
    const planPda = (await generateKeyPairSigner()).address;
    const mint = (await generateKeyPairSigner()).address;
    return {
        merchant,
        planData: {
            destinations: [ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS],
            endTs: 0n,
            metadataUri: '',
            mint,
            planId: 1n,
            pullers: [ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS, ZERO_ADDRESS],
            terms: { amount: 1n, createdAt: 0n, periodHours: 1n },
        },
        planPda,
        tokenMint: mint,
    };
}

describe('createPlan optional payer', () => {
    test('payer is appended as the sixth account meta, writable signer', async () => {
        const inputs = await planInputs();
        const payer = await generateKeyPairSigner();

        const ix = getCreatePlanInstruction({ ...inputs, payer });

        expect(ix.accounts).toHaveLength(6);
        expect(ix.accounts[5].address).toBe(payer.address);
        expect(ix.accounts[5].role).toBe(AccountRole.WRITABLE_SIGNER);
    });

    test('payer is omitted entirely when not provided', async () => {
        const inputs = await planInputs();

        const ix = getCreatePlanInstruction(inputs);

        expect(ix.accounts).toHaveLength(5);
    });
});
