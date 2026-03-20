import { PublicKey } from '@solana/web3.js';

export function validateAddress(value: string, fieldName = 'Address'): string | null {
    const trimmed = value.trim();
    if (!trimmed) return `${fieldName} is required`;
    try {
        void new PublicKey(trimmed);
        return null;
    } catch {
        return `${fieldName} is not a valid Solana address`;
    }
}

export function validateOptionalAddress(value: string, fieldName = 'Address'): string | null {
    if (!value.trim()) return null;
    return validateAddress(value, fieldName);
}

export function validateInteger(value: string, fieldName = 'Value'): string | null {
    if (!value.trim()) return `${fieldName} is required`;
    const n = Number(value.trim());
    if (!Number.isFinite(n) || !Number.isInteger(n)) return `${fieldName} must be an integer`;
    return null;
}

export function parseBigIntValue(value: string): bigint {
    return BigInt(value.trim());
}
