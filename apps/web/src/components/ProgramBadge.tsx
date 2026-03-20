'use client';

import { useEffect, useRef, useState } from 'react';
import { Button } from '@solana/design-system/button';
import { TextInput } from '@solana/design-system/text-input';
import {
    clearStoredProgramAddress, getDefaultProgramAddress, getProgramAddress,
    getStoredProgramAddress, setStoredProgramAddress,
} from '@/lib/program';
import { validateAddress } from '@/lib/validation';

function truncate(value: string, start = 4, end = 4) {
    if (value.length <= start + end + 3) return value;
    return `${value.slice(0, start)}...${value.slice(-end)}`;
}

export function ProgramBadge() {
    const [open, setOpen] = useState(false);
    const [customInput, setCustomInput] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [programId, setProgramId] = useState(getDefaultProgramAddress());
    const [hasCustom, setHasCustom] = useState(false);
    const containerRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        setProgramId(getProgramAddress());
        setHasCustom(Boolean(getStoredProgramAddress()));
    }, []);

    useEffect(() => {
        const handlePointerDown = (e: MouseEvent) => {
            if (!open) return;
            if (!containerRef.current?.contains(e.target as Node)) { setOpen(false); setError(null); }
        };
        const handleEscape = (e: KeyboardEvent) => { if (e.key === 'Escape') { setOpen(false); setError(null); } };
        document.addEventListener('mousedown', handlePointerDown);
        document.addEventListener('keydown', handleEscape);
        return () => { document.removeEventListener('mousedown', handlePointerDown); document.removeEventListener('keydown', handleEscape); };
    }, [open]);

    const apply = () => {
        const err = validateAddress(customInput, 'Program ID');
        if (err) { setError(err); return; }
        const next = setStoredProgramAddress(customInput);
        if (!next) { setError('Not a valid Solana address.'); return; }
        setProgramId(next); setHasCustom(true); setCustomInput(''); setError(null); setOpen(false);
    };

    const reset = () => {
        clearStoredProgramAddress();
        setProgramId(getDefaultProgramAddress());
        setHasCustom(false); setCustomInput(''); setError(null); setOpen(false);
    };

    return (
        <div ref={containerRef} style={{ position: 'relative' }}>
            <Button onClick={() => setOpen(v => !v)} variant="secondary" size="sm"
                style={{ alignItems: 'center', display: 'flex', fontSize: '0.75rem', gap: 4 }}>
                {hasCustom ? `Custom ${truncate(programId)}` : 'Default Program'} ▾
            </Button>
            {open && (
                <div style={{
                    background: 'var(--color-card)', border: '1px solid var(--color-border)',
                    borderRadius: 6, left: 0, minWidth: 360, overflow: 'hidden',
                    padding: 10, position: 'absolute', top: '110%', zIndex: 100,
                }}>
                    <div style={{ color: 'var(--color-muted)', fontSize: '0.75rem', marginBottom: 8 }}>
                        Active: {truncate(programId, 8, 8)}
                    </div>
                    <div style={{ marginBottom: 8 }}>
                        <TextInput value={customInput} onChange={e => setCustomInput(e.target.value)}
                            placeholder="Enter custom program ID" size="md"
                            onKeyDown={e => { if (e.key === 'Enter') { e.preventDefault(); apply(); } }} />
                    </div>
                    {error && <div style={{ color: 'var(--color-error)', fontSize: '0.75rem', marginBottom: 8 }}>{error}</div>}
                    <div style={{ display: 'flex', gap: 8 }}>
                        <Button type="button" size="sm" onClick={apply}>Set Program ID</Button>
                        <Button type="button" size="sm" variant="secondary" onClick={reset}>Use Default</Button>
                    </div>
                </div>
            )}
        </div>
    );
}
