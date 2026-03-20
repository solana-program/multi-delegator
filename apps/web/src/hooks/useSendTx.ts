'use client';

import {
    appendTransactionMessageInstructions,
    assertIsTransactionWithBlockhashLifetime,
    createTransactionMessage,
    getSignatureFromTransaction,
    type Instruction,
    pipe,
    sendAndConfirmTransactionFactory,
    setTransactionMessageFeePayerSigner,
    setTransactionMessageLifetimeUsingBlockhash,
    signTransactionMessageWithSigners,
} from '@solana/kit';
import { useCallback, useState } from 'react';
import type { RecentTransactionValues } from '@/contexts/RecentTransactionsContext';
import { useRecentTransactions } from '@/contexts/RecentTransactionsContext';
import { useWallet } from '@/contexts/WalletContext';
import { formatTransactionError } from '@/lib/transactionErrors';
import { useRpc, useRpcSubscriptions } from './useRpc';

export interface SendTxState {
    error: string | null;
    sending: boolean;
    signature: string | null;
}

export interface SendTxOptions {
    action?: string;
    values?: RecentTransactionValues;
}

export function useSendTx() {
    const rpc = useRpc();
    const rpcSubscriptions = useRpcSubscriptions();
    const { createSigner } = useWallet();
    const { addRecentTransaction } = useRecentTransactions();

    const [state, setState] = useState<SendTxState>({ error: null, sending: false, signature: null });

    const send = useCallback(
        async (instructions: readonly Instruction[], options?: SendTxOptions) => {
            const signer = createSigner();
            if (!signer) {
                setState(s => ({ ...s, error: 'Wallet not connected' }));
                return null;
            }

            setState({ error: null, sending: true, signature: null });

            let txSignature: string | null = null;
            try {
                const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
                const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
                const txId = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

                const txMessage = pipe(
                    createTransactionMessage({ version: 0 }),
                    tx => setTransactionMessageFeePayerSigner(signer, tx),
                    tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
                    tx => appendTransactionMessageInstructions(instructions, tx),
                );

                const signedTx = await signTransactionMessageWithSigners(txMessage);
                txSignature = getSignatureFromTransaction(signedTx);
                assertIsTransactionWithBlockhashLifetime(signedTx);

                await sendAndConfirm(signedTx, { commitment: 'confirmed', skipPreflight: true });

                addRecentTransaction({
                    action: options?.action ?? 'Transaction',
                    id: txId,
                    signature: txSignature,
                    status: 'success',
                    timestamp: Date.now(),
                    values: options?.values,
                });
                setState({ error: null, sending: false, signature: txSignature });
                return txSignature;
            } catch (err) {
                const message = formatTransactionError(err);
                console.error('Transaction failed', err);
                addRecentTransaction({
                    action: options?.action ?? 'Transaction',
                    error: message,
                    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
                    signature: txSignature,
                    status: 'failed',
                    timestamp: Date.now(),
                    values: options?.values,
                });
                setState({ error: message, sending: false, signature: null });
                return null;
            }
        },
        [rpc, rpcSubscriptions, createSigner, addRecentTransaction],
    );

    const reset = useCallback(() => setState({ error: null, sending: false, signature: null }), []);

    return { ...state, reset, send };
}
