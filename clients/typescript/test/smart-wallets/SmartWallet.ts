import type { Wallet } from '../../src/wallet.ts';

export interface SmartWallet extends Wallet {
  readonly name: string;
}
