import type {
  Address,
  GetLatestBlockhashApi,
  Instruction,
  Rpc,
  TransactionSigner,
} from 'gill';
import { createTransaction, signTransactionMessageWithSigners } from 'gill';

export interface Wallet {
  readonly address: Address;
  sendInstructions(instructions: Instruction[]): Promise<string>;
}

type SolanaClient = {
  rpc: Rpc<GetLatestBlockhashApi>;
  sendAndConfirmTransaction: (
    tx: Awaited<ReturnType<typeof signTransactionMessageWithSigners>>,
  ) => Promise<string>;
};

export class KeyPairWallet implements Wallet {
  readonly address: Address;

  constructor(
    private readonly signer: TransactionSigner,
    private readonly client: SolanaClient,
  ) {
    this.address = signer.address;
  }

  async sendInstructions(instructions: Instruction[]): Promise<string> {
    const { value: latestBlockhash } = await this.client.rpc
      .getLatestBlockhash()
      .send();

    const transactionMessage = createTransaction({
      instructions,
      feePayer: this.signer,
      latestBlockhash,
    });
    const signedTransaction =
      await signTransactionMessageWithSigners(transactionMessage);
    return this.client.sendAndConfirmTransaction(signedTransaction);
  }
}

export function addressAsSigner(address: Address): TransactionSigner {
  return { address } as unknown as TransactionSigner;
}
