'use client';

import { useState } from 'react';
import type { Address } from '@solana/kit';
import { buildInitMultiDelegate } from '@multidelegator/client';
import { findAssociatedTokenPda } from '@solana-program/token';
import { useWallet } from '@/contexts/WalletContext';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { getProgramAddress, TOKEN_2022_PROGRAM_ID } from '@/lib/program';
import { useSendTx } from '@/hooks/useSendTx';
import { FormField, SendButton, TxResultDisplay } from './shared';

export function InitMultiDelegate() {
    const { createSigner } = useWallet();
    const { send, sending, error, signature } = useSendTx();
    const { defaultMint } = useSavedValues();

    const [mint, setMint] = useState(defaultMint);
    const [userAta, setUserAta] = useState('');

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const signer = createSigner();
        if (!signer) return;

        const mintAddress = mint.trim() as Address;
        const tokenProgram = TOKEN_2022_PROGRAM_ID;

        let ataAddress: Address;
        if (userAta.trim()) {
            ataAddress = userAta.trim() as Address;
        } else {
            const [derived] = await findAssociatedTokenPda({ mint: mintAddress, owner: signer.address, tokenProgram });
            ataAddress = derived;
        }

        const { instructions, multiDelegatePda } = await buildInitMultiDelegate({
            owner: signer, tokenMint: mintAddress, userAta: ataAddress,
            tokenProgram, programAddress: getProgramAddress(),
        });

        await send(instructions, {
            action: 'InitMultiDelegate',
            values: { mint: mintAddress, multiDelegate: multiDelegatePda },
        });
    }

    return (
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField label="Token Mint" value={mint} onChange={setMint} placeholder="Mint address (Token-2022)" required />
            <FormField label="User ATA (optional)" value={userAta} onChange={setUserAta}
                placeholder="Auto-derived from wallet + mint if empty"
                hint="Leave blank to auto-derive the associated token account" />
            <SendButton sending={sending} />
            <TxResultDisplay signature={signature} error={error} />
        </form>
    );
}
