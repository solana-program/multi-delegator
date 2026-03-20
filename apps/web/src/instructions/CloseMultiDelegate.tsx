'use client';

import { useState } from 'react';
import type { Address } from '@solana/kit';
import { buildCloseMultiDelegate } from '@multidelegator/client';
import { useWallet } from '@/contexts/WalletContext';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { getProgramAddress } from '@/lib/program';
import { useSendTx } from '@/hooks/useSendTx';
import { FormField, SendButton, TxResultDisplay } from './shared';

export function CloseMultiDelegate() {
    const { createSigner } = useWallet();
    const { send, sending, error, signature } = useSendTx();
    const { defaultMint } = useSavedValues();

    const [mint, setMint] = useState(defaultMint);

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const signer = createSigner();
        if (!signer) return;

        const { instructions } = await buildCloseMultiDelegate({
            user: signer, tokenMint: mint.trim() as Address, programAddress: getProgramAddress(),
        });

        await send(instructions, { action: 'CloseMultiDelegate', values: { mint: mint.trim() } });
    }

    return (
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField label="Token Mint" value={mint} onChange={setMint} placeholder="Mint address" required />
            <SendButton sending={sending} />
            <TxResultDisplay signature={signature} error={error} />
        </form>
    );
}
