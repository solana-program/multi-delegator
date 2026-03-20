'use client';

import { createSolanaRpc, createSolanaRpcSubscriptions } from '@solana/kit';
import { useMemo } from 'react';
import { useRpcContext } from '@/contexts/RpcContext';

function wsUrlFromHttp(httpUrl: string): string {
    return httpUrl.replace(/^https?:\/\//, match => (match === 'https://' ? 'wss://' : 'ws://'));
}

export function useRpc() {
    const { rpcUrl } = useRpcContext();
    return useMemo(() => createSolanaRpc(rpcUrl), [rpcUrl]);
}

export function useRpcSubscriptions() {
    const { rpcUrl } = useRpcContext();
    return useMemo(() => createSolanaRpcSubscriptions(wsUrlFromHttp(rpcUrl)), [rpcUrl]);
}
