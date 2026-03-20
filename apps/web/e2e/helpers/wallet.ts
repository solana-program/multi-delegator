import type { Page } from '@playwright/test';

/**
 * Injects a mock Phantom wallet into the page using TweetNaCl for Ed25519 signing.
 *
 * Must be called after page.goto() but before clicking "Select Wallet".
 * After calling this, call connectWallet() to trigger the adapter connect flow.
 *
 * Returns the wallet's base58 public key.
 */
export async function injectWallet(page: Page, walletKeyBase58: string): Promise<string> {
    await page.evaluate(key => {
        (window as any)._walletKey = key;
    }, walletKeyBase58);

    await page.evaluate(
        () =>
            new Promise<void>((resolve, reject) => {
                const script = document.createElement('script');
                script.src = 'https://cdn.jsdelivr.net/npm/tweetnacl@1.0.3/nacl-fast.min.js';
                script.onload = () => resolve();
                script.onerror = () => reject(new Error('Failed to load TweetNaCl'));
                document.head.appendChild(script);
            }),
    );

    // Minimal Buffer polyfill — the Phantom wallet adapter uses Buffer.from() internally.
    await page.evaluate(() => {
        (window as any).Buffer = {
            alloc: (size: number, fill = 0) => new Uint8Array(size).fill(fill),
            concat: (bufs: Uint8Array[]) => {
                const total = bufs.reduce((s, b) => s + b.length, 0);
                const result = new Uint8Array(total);
                let offset = 0;
                for (const b of bufs) {
                    result.set(b, offset);
                    offset += b.length;
                }
                return result;
            },
            from: (data: any) => {
                if (data instanceof Uint8Array) return data;
                if (Array.isArray(data)) return new Uint8Array(data);
                return new Uint8Array(data);
            },
            isBuffer: (obj: any) => obj instanceof Uint8Array,
        };
    });

    const pubkey = await page.evaluate((walletKey: string) => {
        const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

        function b58Decode(s: string): Uint8Array {
            const bytes = [0];
            for (const c of s) {
                const idx = ALPHABET.indexOf(c);
                if (idx < 0) throw new Error('Invalid base58 char: ' + c);
                let carry = idx;
                for (let j = 0; j < bytes.length; j++) {
                    carry += bytes[j] * 58;
                    bytes[j] = carry & 0xff;
                    carry >>= 8;
                }
                while (carry > 0) {
                    bytes.push(carry & 0xff);
                    carry >>= 8;
                }
            }
            for (const c of s) {
                if (c === '1') bytes.push(0);
                else break;
            }
            return new Uint8Array(bytes.reverse());
        }

        function b58Encode(bytes: Uint8Array): string {
            const digits = [0];
            for (let i = 0; i < bytes.length; i++) {
                let carry = bytes[i];
                for (let j = 0; j < digits.length; j++) {
                    carry += digits[j] * 256;
                    digits[j] = carry % 58;
                    carry = Math.floor(carry / 58);
                }
                while (carry > 0) {
                    digits.push(carry % 58);
                    carry = Math.floor(carry / 58);
                }
            }
            let result = '';
            for (let i = 0; i < bytes.length - 1 && bytes[i] === 0; i++) result += '1';
            return (
                result +
                digits
                    .reverse()
                    .map(d => ALPHABET[d])
                    .join('')
            );
        }

        const nacl = (window as any).nacl;
        const kp = nacl.sign.keyPair.fromSecretKey(b58Decode(walletKey));
        const pubkeyB58 = b58Encode(kp.publicKey);

        (window as any)._kp = kp;
        (window as any)._pubkey = pubkeyB58;

        (window as any).solana = {
            _events: {} as Record<string, ((...args: any[]) => void)[]>,
            connect: async () => ({ publicKey: (window as any).solana.publicKey }),
            disconnect: async () => {},
            emit(event: string, ...args: any[]) {
                (this._events[event] ?? []).forEach((h: any) => h(...args));
            },
            isConnected: true,
            isPhantom: true,
            off(event: string, handler: (...args: any[]) => void) {
                if (this._events[event]) {
                    this._events[event] = this._events[event].filter((h: any) => h !== handler);
                }
            },
            on(event: string, handler: (...args: any[]) => void) {
                if (!this._events[event]) this._events[event] = [];
                this._events[event].push(handler);
            },
            publicKey: {
                toBase58: () => pubkeyB58,
                toBytes: () => kp.publicKey,
                toString: () => pubkeyB58,
            },
            removeListener(event: string, handler: (...args: any[]) => void) {
                this.off(event, handler);
            },
            signAllTransactions: async (txs: any[]) =>
                await Promise.all(txs.map((tx: any) => (window as any).solana.signTransaction(tx))),
            signMessage: async (msg: Uint8Array) => ({
                signature: new Uint8Array(nacl.sign.detached(msg, kp.secretKey)),
            }),
            signTransaction: async (tx: any) => {
                const msgBytes = new Uint8Array(tx.message.serialize());
                const sig = nacl.sign.detached(msgBytes, kp.secretKey);
                tx.signatures[0] = new Uint8Array(sig);
                return tx;
            },
        };

        return pubkeyB58;
    }, walletKeyBase58);

    return pubkey;
}

/**
 * Opens the wallet modal and selects "Phantom Detected".
 *
 * Must be called after injectWallet(). The adapter captures window.solana.signTransaction
 * at connect time, so this must happen after injection — not before.
 */
export async function connectWallet(page: Page): Promise<void> {
    const connectBtn = page.getByRole('button', { name: /Select Wallet|Connect Wallet/ });
    await connectBtn.click();
    await page.getByRole('button', { name: /Phantom.*Detected/i }).click();
    await page.getByRole('button', { name: /Disconnect/i }).waitFor({ timeout: 8000 });
}
