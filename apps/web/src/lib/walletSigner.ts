'use client';

import { getTransactionDecoder, getTransactionEncoder, type Address, type TransactionSigner } from '@solana/kit';
import { VersionedTransaction, type Transaction } from '@solana/web3.js';

type WalletAdapterSignTransaction = (transaction: VersionedTransaction) => Promise<Transaction | VersionedTransaction>;

export function createWalletAdapterTransactionSigner(
    walletAddress: Address,
    signTransaction: WalletAdapterSignTransaction,
): TransactionSigner {
    const encoder = getTransactionEncoder();
    const decoder = getTransactionDecoder();

    return {
        address: walletAddress,
        signTransactions: async transactions => {
            return Promise.all(
                transactions.map(async tx => {
                    const bytes = new Uint8Array(encoder.encode(tx));
                    const versionedTransaction = VersionedTransaction.deserialize(bytes);
                    const signedTransaction = await signTransaction(versionedTransaction);
                    const signedBytes = signedTransaction.serialize();
                    const decodedTransaction = decoder.decode(new Uint8Array(signedBytes)) as typeof tx & {
                        signatures: Record<string, Uint8Array | null>;
                    };

                    const walletSignature = decodedTransaction.signatures[walletAddress];
                    if (!walletSignature) {
                        throw new Error(`Wallet did not return a signature for ${walletAddress}`);
                    }

                    return Object.freeze({ [walletAddress]: walletSignature });
                }),
            );
        },
    };
}
