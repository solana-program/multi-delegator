'use client';

import { Badge } from '@solana/design-system/badge';
import { Button } from '@solana/design-system/button';
import { useRpcContext } from '@/contexts/RpcContext';
import { getClusterFromRpcUrl, getSolanaExplorerUrl } from '@/lib/explorer';

interface TxResultProps {
    signature: string | null;
    error: string | null;
}

export function TxResult({ signature, error }: TxResultProps) {
    const { rpcUrl } = useRpcContext();

    if (!signature && !error) return null;

    const cluster = getClusterFromRpcUrl(rpcUrl);

    if (error) {
        return (
            <div style={{
                marginTop: 16, padding: '10px 12px', borderRadius: 6,
                border: '1px solid var(--status-error-border)',
                background: 'var(--status-error-bg)',
                color: 'var(--status-error-text)',
                fontSize: '0.8125rem', wordBreak: 'break-all',
                display: 'flex', alignItems: 'center', gap: 8,
            }}>
                <Badge variant="danger">Failed</Badge>
                <span>{error}</span>
            </div>
        );
    }

    if (signature) {
        const explorerUrl = getSolanaExplorerUrl(signature, cluster);
        return (
            <div style={{
                marginTop: 16, padding: '10px 12px', borderRadius: 6,
                border: '1px solid var(--status-success-border)',
                background: 'var(--status-success-bg)',
                display: 'flex', alignItems: 'center', gap: 12, fontSize: '0.8125rem',
            }}>
                <Badge variant="success">Success</Badge>
                <span style={{ color: 'var(--status-success-text)' }}>tx: {signature.slice(0, 8)}...</span>
                <Button asChild size="sm" variant="secondary">
                    <a href={explorerUrl} target="_blank" rel="noopener noreferrer">View on Explorer</a>
                </Button>
            </div>
        );
    }

    return null;
}
