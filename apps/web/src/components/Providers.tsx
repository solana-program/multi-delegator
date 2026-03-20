'use client';

import type { ReactNode } from 'react';
import { RpcProvider } from '@/contexts/RpcContext';
import { WalletProvider } from '@/contexts/WalletContext';
import { RecentTransactionsProvider } from '@/contexts/RecentTransactionsContext';
import { SavedValuesProvider } from '@/contexts/SavedValuesContext';

export function Providers({ children }: { children: ReactNode }) {
    return (
        <RpcProvider>
            <WalletProvider>
                <SavedValuesProvider>
                    <RecentTransactionsProvider>{children}</RecentTransactionsProvider>
                </SavedValuesProvider>
            </WalletProvider>
        </RpcProvider>
    );
}
