'use client';

import { createContext, useCallback, useContext, useMemo } from 'react';
import type { Address, TransactionSigner } from '@solana/kit';
import {
    ConnectionProvider,
    WalletProvider as AdapterWalletProvider,
    useWallet as useAdapterWallet,
} from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { PhantomWalletAdapter, SolflareWalletAdapter } from '@solana/wallet-adapter-wallets';
import { useRpcContext } from '@/contexts/RpcContext';
import { createWalletAdapterTransactionSigner } from '@/lib/walletSigner';

interface WalletAccount {
    address: string;
}

interface WalletContextType {
    account: WalletAccount | null;
    connected: boolean;
    connecting: boolean;
    createSigner: () => TransactionSigner | null;
}

const WalletContext = createContext<WalletContextType | null>(null);

function WalletStateProvider({ children }: { children: React.ReactNode }) {
    const { publicKey, connected, connecting, signTransaction } = useAdapterWallet();

    const account = useMemo<WalletAccount | null>(
        () => (publicKey ? { address: publicKey.toBase58() } : null),
        [publicKey],
    );

    const signer = useMemo<TransactionSigner | null>(() => {
        if (!publicKey || !signTransaction) return null;
        return createWalletAdapterTransactionSigner(publicKey.toBase58() as Address, tx => signTransaction(tx));
    }, [publicKey, signTransaction]);

    const createSigner = useCallback((): TransactionSigner | null => signer, [signer]);

    const value = useMemo(
        () => ({ account, connected, connecting, createSigner }),
        [account, connected, connecting, createSigner],
    );

    return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function WalletProvider({ children }: { children: React.ReactNode }) {
    const { rpcUrl } = useRpcContext();
    const wallets = useMemo(() => [new PhantomWalletAdapter(), new SolflareWalletAdapter()], []);

    return (
        <ConnectionProvider endpoint={rpcUrl}>
            <AdapterWalletProvider wallets={wallets} autoConnect>
                <WalletModalProvider>
                    <WalletStateProvider>{children}</WalletStateProvider>
                </WalletModalProvider>
            </AdapterWalletProvider>
        </ConnectionProvider>
    );
}

export function useWallet() {
    const ctx = useContext(WalletContext);
    if (!ctx) throw new Error('useWallet must be used inside WalletProvider');
    return ctx;
}
