import type {
  GetAccountInfoApi,
  GetLatestBlockhashApi,
  GetProgramAccountsApi,
  Rpc,
  signTransactionMessageWithSigners,
} from 'gill';

/** Bundles an RPC connection and a transaction sender for use throughout the SDK. */
export type SolanaClient = {
  rpc: Rpc<GetAccountInfoApi & GetLatestBlockhashApi & GetProgramAccountsApi>;
  sendAndConfirmTransaction: (
    tx: Awaited<ReturnType<typeof signTransactionMessageWithSigners>>,
  ) => Promise<string>;
};

/** Result of a successfully sent transaction. */
export type TransactionResult = { signature: string };
