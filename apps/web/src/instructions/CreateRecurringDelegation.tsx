'use client';

import { useState } from 'react';
import type { Address } from '@solana/kit';
import { buildCreateRecurringDelegation } from '@multidelegator/client';
import { useWallet } from '@/contexts/WalletContext';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { getProgramAddress } from '@/lib/program';
import { useSendTx } from '@/hooks/useSendTx';
import { FormField, SendButton, TxResultDisplay } from './shared';

export function CreateRecurringDelegation() {
    const { createSigner } = useWallet();
    const { send, sending, error, signature } = useSendTx();
    const { defaultMint, defaultDelegatee } = useSavedValues();

    const [mint, setMint] = useState(defaultMint);
    const [delegatee, setDelegatee] = useState(defaultDelegatee);
    const [nonce, setNonce] = useState('0');
    const [amountPerPeriod, setAmountPerPeriod] = useState('');
    const [periodLengthS, setPeriodLengthS] = useState('');
    const [expiryTs, setExpiryTs] = useState('0');
    const [startTs, setStartTs] = useState('0');

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const signer = createSigner();
        if (!signer) return;

        const { instructions, delegationPda } = await buildCreateRecurringDelegation({
            delegator: signer,
            tokenMint: mint.trim() as Address,
            delegatee: delegatee.trim() as Address,
            nonce: BigInt(nonce),
            amountPerPeriod: BigInt(amountPerPeriod),
            periodLengthS: BigInt(periodLengthS),
            startTs: BigInt(startTs),
            expiryTs: BigInt(expiryTs),
            programAddress: getProgramAddress(),
        });

        await send(instructions, {
            action: 'CreateRecurringDelegation',
            values: { mint: mint.trim(), delegatee: delegatee.trim(), delegationPda },
        });
    }

    return (
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField label="Token Mint" value={mint} onChange={setMint} placeholder="Mint address" required />
            <FormField label="Delegatee" value={delegatee} onChange={setDelegatee} placeholder="Delegatee address" required />
            <FormField label="Nonce" value={nonce} onChange={setNonce} type="number" hint="Unique nonce" required />
            <FormField label="Amount Per Period" value={amountPerPeriod} onChange={setAmountPerPeriod} type="number" hint="Token amount per period (base units)" required />
            <FormField label="Period Length (seconds)" value={periodLengthS} onChange={setPeriodLengthS} type="number" hint="e.g. 86400 for 1 day" required />
            <FormField label="Expiry Timestamp" value={expiryTs} onChange={setExpiryTs} type="number" hint="Unix timestamp (0 = no expiry)" required />
            <FormField label="Start Timestamp" value={startTs} onChange={setStartTs} type="number" hint="Unix timestamp (0 = immediate)" required />
            <SendButton sending={sending} />
            <TxResultDisplay signature={signature} error={error} />
        </form>
    );
}
