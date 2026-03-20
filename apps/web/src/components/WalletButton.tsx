'use client';

import { useCallback } from 'react';
import { Button } from '@solana/design-system/button';
import { useWallet as useAdapterWallet } from '@solana/wallet-adapter-react';
import { useWalletModal } from '@solana/wallet-adapter-react-ui';

function truncateAddress(address: string, start = 6, end = 6) {
    if (address.length <= start + end + 3) return address;
    return `${address.slice(0, start)}...${address.slice(-end)}`;
}

export function WalletButton() {
    const adapterWallet = useAdapterWallet();
    const { wallet, publicKey, connected, connecting } = adapterWallet;
    const { setVisible } = useWalletModal();

    const handleConnect = useCallback(async () => {
        if (!wallet) { setVisible(true); return; }
        try { await adapterWallet.connect(); } catch { /* silent */ }
    }, [wallet, setVisible, adapterWallet]);

    if (connected && publicKey) {
        return (
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <Button type="button" size="sm" variant="secondary" onClick={() => setVisible(true)}>
                    {truncateAddress(publicKey.toBase58())}
                </Button>
                <Button type="button" size="sm" variant="secondary" onClick={() => void adapterWallet.disconnect()}>
                    Disconnect
                </Button>
            </div>
        );
    }

    return (
        <Button type="button" size="sm" loading={connecting} disabled={connecting} onClick={() => void handleConnect()}>
            {wallet ? 'Connect Wallet' : 'Select Wallet'}
        </Button>
    );
}
