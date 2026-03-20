'use client';

import { useState } from 'react';
import type { Address } from '@solana/kit';
import { buildRevokeDelegation } from '@multidelegator/client';
import { useWallet } from '@/contexts/WalletContext';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { getProgramAddress } from '@/lib/program';
import { useSendTx } from '@/hooks/useSendTx';
import { FormField, SendButton, TxResultDisplay } from './shared';

export function RevokeDelegation() {
    const { createSigner } = useWallet();
    const { send, sending, error, signature } = useSendTx();
    const { defaultDelegation } = useSavedValues();

    const [delegationAccount, setDelegationAccount] = useState(defaultDelegation);

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const signer = createSigner();
        if (!signer) return;

        const { instructions } = buildRevokeDelegation({
            authority: signer,
            delegationAccount: delegationAccount.trim() as Address,
            programAddress: getProgramAddress(),
        });

        await send(instructions, {
            action: 'RevokeDelegation',
            values: { delegationPda: delegationAccount.trim() },
        });
    }

    return (
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField label="Delegation Account" value={delegationAccount} onChange={setDelegationAccount}
                placeholder="Delegation PDA address" required />
            <SendButton sending={sending} />
            <TxResultDisplay signature={signature} error={error} />
        </form>
    );
}
