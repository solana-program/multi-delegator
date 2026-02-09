import type { Address, Instruction } from 'gill';

export interface SmartWallet {
  readonly name: string;
  readonly address: Address;
  sendInstructions(instructions: Instruction[]): Promise<string>;
}
