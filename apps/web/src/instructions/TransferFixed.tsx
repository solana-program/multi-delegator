'use client';

import { useState } from 'react';
import type { Address } from '@solana/kit';
import { buildTransferFixed } from '@multidelegator/client';
import { findAssociatedTokenPda } from '@solana-program/token';
import { useWallet } from '@/contexts/WalletContext';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { getProgramAddress, TOKEN_2022_PROGRAM_ID } from '@/lib/program';
import { useSendTx } from '@/hooks/useSendTx';
import { FormField, SendButton, TxResultDisplay } from './shared';

export function TransferFixed() {
    const { createSigner } = useWallet();
    const { send, sending, error, signature } = useSendTx();
    const { defaultDelegation, defaultMint } = useSavedValues();

    const [delegationPda, setDelegationPda] = useState(defaultDelegation);
    const [delegator, setDelegator] = useState('');
    const [tokenMint, setTokenMint] = useState(defaultMint);
    const [amount, setAmount] = useState('');
    const [receiverAta, setReceiverAta] = useState('');

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const signer = createSigner();
        if (!signer) return;

        const mintAddress = tokenMint.trim() as Address;
        const delegatorAddress = delegator.trim() as Address;
        const tokenProgram = TOKEN_2022_PROGRAM_ID;

        const [delegatorAta] = await findAssociatedTokenPda({ mint: mintAddress, owner: delegatorAddress, tokenProgram });

        let receiver: Address;
        if (receiverAta.trim()) {
            receiver = receiverAta.trim() as Address;
        } else {
            const [derived] = await findAssociatedTokenPda({ mint: mintAddress, owner: signer.address, tokenProgram });
            receiver = derived;
        }

        const { instructions } = await buildTransferFixed({
            delegatee: signer, delegator: delegatorAddress, delegatorAta,
            tokenMint: mintAddress, delegationPda: delegationPda.trim() as Address,
            amount: BigInt(amount), receiverAta: receiver, tokenProgram,
            programAddress: getProgramAddress(),
        });

        await send(instructions, {
            action: 'TransferFixed',
            values: { delegationPda: delegationPda.trim(), mint: mintAddress, amount },
        });
    }

    return (
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField label="Delegation PDA" value={delegationPda} onChange={setDelegationPda} placeholder="Delegation account address" required />
            <FormField label="Delegator" value={delegator} onChange={setDelegator} placeholder="Delegator wallet address" required />
            <FormField label="Token Mint" value={tokenMint} onChange={setTokenMint} placeholder="Mint address" required />
            <FormField label="Amount" value={amount} onChange={setAmount} type="number" hint="Amount to transfer (base units)" required />
            <FormField label="Receiver ATA (optional)" value={receiverAta} onChange={setReceiverAta}
                placeholder="Auto-derived from connected wallet + mint if empty" />
            <SendButton sending={sending} />
            <TxResultDisplay signature={signature} error={error} />
        </form>
    );
}
