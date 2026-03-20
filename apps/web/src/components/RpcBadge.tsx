'use client';

import { useEffect, useRef, useState } from 'react';
import { Button } from '@solana/design-system/button';
import { TextInput } from '@solana/design-system/text-input';
import { RPC_PRESETS, useRpcContext } from '@/contexts/RpcContext';
import { getClusterFromRpcUrl } from '@/lib/explorer';

export function RpcBadge() {
    const { rpcUrl, setRpcUrl } = useRpcContext();
    const [open, setOpen] = useState(false);
    const [customInput, setCustomInput] = useState('');
    const containerRef = useRef<HTMLDivElement | null>(null);

    const label = RPC_PRESETS.find(p => p.url === rpcUrl)?.label ?? 'Custom';

    useEffect(() => {
        const handlePointerDown = (e: MouseEvent) => {
            if (!open) return;
            if (!containerRef.current?.contains(e.target as Node)) setOpen(false);
        };
        const handleEscape = (e: KeyboardEvent) => { if (e.key === 'Escape') setOpen(false); };
        document.addEventListener('mousedown', handlePointerDown);
        document.addEventListener('keydown', handleEscape);
        return () => {
            document.removeEventListener('mousedown', handlePointerDown);
            document.removeEventListener('keydown', handleEscape);
        };
    }, [open]);

    return (
        <div ref={containerRef} style={{ position: 'relative' }}>
            <Button onClick={() => setOpen(v => !v)} variant="secondary" size="sm"
                style={{ fontSize: '0.75rem', display: 'flex', alignItems: 'center', gap: 4 }}>
                {label} ▾
            </Button>
            {open && (
                <div style={{
                    position: 'absolute', top: '110%', left: 0,
                    background: 'var(--color-card)', border: '1px solid var(--color-border)',
                    borderRadius: 6, minWidth: 240, zIndex: 100, overflow: 'hidden',
                }}>
                    {RPC_PRESETS.map(preset => (
                        <Button key={preset.url}
                            onClick={() => { setRpcUrl(preset.url); setOpen(false); }}
                            variant={rpcUrl === preset.url ? 'primary' : 'secondary'} size="sm"
                            style={{ width: '100%', justifyContent: 'space-between', borderRadius: 0, fontSize: '0.8125rem' }}>
                            <span>{preset.label}</span>
                            <span style={{ color: 'var(--color-muted)', fontSize: '0.75rem' }}>
                                {getClusterFromRpcUrl(preset.url)}
                            </span>
                        </Button>
                    ))}
                    <div style={{ borderTop: '1px solid var(--color-border)', padding: '8px 10px', display: 'flex', gap: 6 }}>
                        <div style={{ flex: 1 }}>
                            <TextInput value={customInput} onChange={e => setCustomInput(e.target.value)}
                                placeholder="https://my-rpc.com" size="md"
                                onKeyDown={e => { if (e.key === 'Enter' && customInput) { setRpcUrl(customInput); setCustomInput(''); setOpen(false); } }} />
                        </div>
                        <Button onClick={() => { if (customInput) { setRpcUrl(customInput); setCustomInput(''); setOpen(false); } }} size="sm">Set</Button>
                    </div>
                </div>
            )}
        </div>
    );
}
