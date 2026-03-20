'use client';

import type { ReactNode } from 'react';
import { Button } from '@solana/design-system/button';
import { TextInput } from '@solana/design-system/text-input';
import { TxResult } from '@/components/TxResult';

interface FormFieldProps {
    label: string;
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    hint?: string;
    required?: boolean;
    readOnly?: boolean;
    type?: string;
    autoFillValue?: string;
    onAutoFill?: (v: string) => void;
    autoFillLabel?: string;
}

export function FormField({
    label, value, onChange, placeholder, hint, required, readOnly,
    type = 'text', autoFillValue = '', onAutoFill, autoFillLabel = 'Autofill',
}: FormFieldProps) {
    return (
        <TextInput label={label} description={hint}
            action={onAutoFill ? (
                <Button type="button" size="sm" variant="secondary"
                    onClick={() => onAutoFill(autoFillValue)} disabled={!autoFillValue}>
                    {autoFillLabel}
                </Button>
            ) : undefined}
            type={type} value={value} onChange={e => onChange(e.target.value)}
            placeholder={placeholder} required={required} readOnly={readOnly}
        />
    );
}

interface SelectFieldProps {
    label: string;
    value: string;
    onChange: (v: string) => void;
    options: { label: string; value: string }[];
    hint?: string;
}

export function SelectField({ label, value, onChange, options, hint }: SelectFieldProps) {
    return (
        <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            <span style={{ fontSize: '0.8125rem', fontWeight: 500 }}>{label}</span>
            {hint && <span style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>{hint}</span>}
            <select value={value} onChange={e => onChange(e.target.value)}
                style={{
                    background: 'var(--color-bg)', border: '1px solid var(--color-border)',
                    color: 'var(--color-text)', borderRadius: 8, padding: '10px 12px',
                    fontSize: '0.8125rem',
                }}>
                {options.map(opt => <option key={opt.value} value={opt.value}>{opt.label}</option>)}
            </select>
        </label>
    );
}

export function SendButton({ sending }: { sending: boolean }) {
    return (
        <Button type="submit" loading={sending} disabled={sending} style={{ marginTop: 8, alignSelf: 'flex-start' }}>
            {sending ? 'Sending Transaction' : 'Send Transaction'}
        </Button>
    );
}

interface TxResultDisplayProps {
    signature: string | null;
    error: string | null;
    children?: ReactNode;
}

export function TxResultDisplay({ signature, error }: TxResultDisplayProps) {
    return <TxResult signature={signature} error={error} />;
}
