'use client';

import { useState } from 'react';
import type { Address } from '@solana/kit';
import { buildCreateFixedDelegation } from '@multidelegator/client';
import { useWallet } from '@/contexts/WalletContext';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { getProgramAddress } from '@/lib/program';
import { useSendTx } from '@/hooks/useSendTx';
import { FormField, SendButton, TxResultDisplay } from './shared';

export function CreateFixedDelegation() {
    const { createSigner } = useWallet();
    const { send, sending, error, signature } = useSendTx();
    const { defaultMint, defaultDelegatee } = useSavedValues();

    const [mint, setMint] = useState(defaultMint);
    const [delegatee, setDelegatee] = useState(defaultDelegatee);
    const [nonce, setNonce] = useState('0');
    const [amount, setAmount] = useState('');
    const [expiryTs, setExpiryTs] = useState('0');

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const signer = createSigner();
        if (!signer) return;

        const { instructions, delegationPda } = await buildCreateFixedDelegation({
            delegator: signer,
            tokenMint: mint.trim() as Address,
            delegatee: delegatee.trim() as Address,
            nonce: BigInt(nonce),
            amount: BigInt(amount),
            expiryTs: BigInt(expiryTs),
            programAddress: getProgramAddress(),
        });

        await send(instructions, {
            action: 'CreateFixedDelegation',
            values: { mint: mint.trim(), delegatee: delegatee.trim(), delegationPda },
        });
    }

    return (
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField label="Token Mint" value={mint} onChange={setMint} placeholder="Mint address" required />
            <FormField label="Delegatee" value={delegatee} onChange={setDelegatee} placeholder="Delegatee address" required />
            <FormField label="Nonce" value={nonce} onChange={setNonce} type="number" hint="Unique nonce for this delegation to the same delegatee" required />
            <FormField label="Amount" value={amount} onChange={setAmount} type="number" hint="Total token amount (base units)" required />
            <FormField label="Expiry Timestamp" value={expiryTs} onChange={setExpiryTs} type="number" hint="Unix timestamp (0 = no expiry)" required />
            <SendButton sending={sending} />
            <TxResultDisplay signature={signature} error={error} />
        </form>
    );
}
