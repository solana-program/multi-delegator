import { Connection, Keypair as Web3Keypair } from '@solana/web3.js';
import { getCreateAccountInstruction } from '@solana-program/system';
import {
  findAssociatedTokenPda,
  getCreateAssociatedTokenInstructionAsync,
  getInitializeMintInstruction,
  getMintSize,
  getMintToInstruction,
  TOKEN_PROGRAM_ADDRESS,
} from '@solana-program/token';
import {
  type Address,
  appendTransactionMessageInstructions,
  createSolanaClient,
  createTransactionMessage,
  generateKeyPairSigner,
  type KeyPairSigner,
  lamports,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from 'gill';
import { MultiDelegatorClient } from '../src/client.js';
import {
  createSquadsWallet,
  createSwigWallet,
  type SmartWallet,
} from './smart-wallets/index.ts';

export const SURFPOOL_PORT = 8899;
export const SURFPOOL_RPC_URL = `http://127.0.0.1:${SURFPOOL_PORT}`;
export const DEFAULT_TEST_BALANCE = 1_000_000n;
export const ONE_HOUR_IN_SECONDS = 3600;
export const ONE_DAY_IN_SECONDS = 86400;

type SolanaClient = ReturnType<typeof createSolanaClient>;

/**
 * IntegrationTest class that provides test fixtures and helper methods
 * for integration testing the MultiDelegator program.
 */
export class IntegrationTest {
  /** The MultiDelegatorClient instance for interacting with the program */
  public readonly client: MultiDelegatorClient;

  /** Direct RPC access for queries and assertions */
  public readonly rpc: SolanaClient['rpc'];

  /** Pre-funded keypair signer (10 SOL), also the mint authority for tokenMint */
  public readonly payer: KeyPairSigner;

  /** Pre-created SPL token mint (6 decimals, payer is mint authority) */
  public readonly tokenMint: Address;

  private solanaClient: SolanaClient;

  private constructor(
    solanaClient: SolanaClient,
    client: MultiDelegatorClient,
    payer: KeyPairSigner,
    tokenMint: Address,
  ) {
    this.solanaClient = solanaClient;
    this.client = client;
    this.rpc = solanaClient.rpc;
    this.payer = payer;
    this.tokenMint = tokenMint;
  }

  /**
   * Factory method to create a new IntegrationTest instance.
   * This initializes a payer with 10 SOL and creates a default token mint.
   */
  static async create(): Promise<IntegrationTest> {
    await isSurfnetRunning(); // Just verify surfpool is running
    const solanaClient = createSolanaClient({ urlOrMoniker: 'localnet' });
    const client = new MultiDelegatorClient(solanaClient);

    // Create and fund payer with 10 SOL
    const payer = await createFundedKeypair(solanaClient, 10_000_000_000n);

    // Create default token mint (payer is mint authority)
    const tokenMint = await createMint(solanaClient, payer, 6);

    return new IntegrationTest(solanaClient, client, payer, tokenMint);
  }

  /**
   * Creates a new token mint with the payer as the mint authority.
   * @param decimals - Number of decimals for the mint (default: 6)
   * @returns The address of the newly created mint
   */
  async createTokenMint(decimals: number = 6): Promise<Address> {
    return createMint(this.solanaClient, this.payer, decimals);
  }

  /**
   * Creates an Associated Token Account for the given owner and mints tokens to it.
   * @param mint - The token mint address
   * @param owner - The owner of the ATA
   * @param amount - The amount of tokens to mint (in base units)
   * @returns The address of the created ATA
   */
  async createAtaWithBalance(
    mint: Address,
    owner: Address,
    amount: bigint,
  ): Promise<Address> {
    return createAtaWithTokens(
      this.solanaClient,
      this.payer,
      mint,
      owner,
      amount,
    );
  }

  async createFundedKeypair(
    lamportsAmount: bigint = 1_000_000_000n,
  ): Promise<KeyPairSigner> {
    return createFundedKeypair(this.solanaClient, lamportsAmount);
  }

  async airdropToAddress(
    address: Address,
    lamportsAmount: bigint = 1_000_000_000n,
  ): Promise<void> {
    await airdropToAddress(this.solanaClient, address, lamportsAmount);
  }

  async createSmartWallets(): Promise<SmartWallet[]> {
    const rawChoice = (process.env.SMART_WALLET ?? 'both').toLowerCase();
    if (!['swig', 'squads', 'both'].includes(rawChoice)) {
      throw new Error(
        `Invalid SMART_WALLET value: ${rawChoice}. Use swig, squads, or both.`,
      );
    }
    const choice = rawChoice as 'swig' | 'squads' | 'both';

    const connection = new Connection(SURFPOOL_RPC_URL, 'confirmed');
    const wallets: SmartWallet[] = [];

    if (choice === 'swig' || choice === 'both') {
      const swigFeePayer = Web3Keypair.generate();
      const swigAuthority = Web3Keypair.generate();
      await this.airdropToAddress(
        swigFeePayer.publicKey.toBase58() as Address,
        2_000_000_000n,
      );
      await this.airdropToAddress(
        swigAuthority.publicKey.toBase58() as Address,
        2_000_000_000n,
      );
      const swigWallet = await createSwigWallet({
        connection,
        feePayer: swigFeePayer,
        authority: swigAuthority,
      });
      await this.airdropToAddress(swigWallet.address, 5_000_000_000n);
      wallets.push(swigWallet);
    }

    if (choice === 'squads' || choice === 'both') {
      const squadsMembers = [
        Web3Keypair.generate(),
        Web3Keypair.generate(),
        Web3Keypair.generate(),
      ];
      const squadsFeePayer = squadsMembers[0];
      await this.airdropToAddress(
        squadsFeePayer.publicKey.toBase58() as Address,
        2_000_000_000n,
      );
      for (const member of squadsMembers.slice(1)) {
        await this.airdropToAddress(
          member.publicKey.toBase58() as Address,
          1_000_000_000n,
        );
      }
      const squadsWallet = await createSquadsWallet({
        connection,
        feePayer: squadsFeePayer,
        members: squadsMembers,
        approvalsRequired: 2,
      });
      await this.airdropToAddress(squadsWallet.address, 5_000_000_000n);
      wallets.push(squadsWallet);
    }

    if (wallets.length === 0) {
      throw new Error('No smart wallets were created.');
    }

    return wallets;
  }
}

// ============================================================================
// Private Helper Functions
// ============================================================================

/**
 * Checks that Surfpool is running and returns the RPC URL.
 *
 * Note: Surfpool should be started separately with `surfpool start`
 * which will auto-deploy the multi_delegator program via the runbook.
 *
 * @throws Error if Surfpool is not running
 */
async function isSurfnetRunning(): Promise<string> {
  try {
    const response = await fetch(SURFPOOL_RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'getHealth',
      }),
    });

    if (response.ok) {
      const data = (await response.json()) as { result?: string };
      if (data.result === 'ok') {
        return SURFPOOL_RPC_URL;
      }
    }

    throw new Error('Surfpool returned unhealthy status');
  } catch (_error) {
    throw new Error(
      `Surfpool is not running at ${SURFPOOL_RPC_URL}. Please start it with: surfpool start`,
    );
  }
}

/**
 * Creates a new keypair and funds it with SOL via airdrop.
 */
async function createFundedKeypair(
  client: SolanaClient,
  lamportsAmount: bigint,
): Promise<KeyPairSigner> {
  const keypair = await generateKeyPairSigner();

  // Request airdrop
  const signature = await client.rpc
    .requestAirdrop(keypair.address, lamports(lamportsAmount))
    .send();

  // Poll until confirmed
  let confirmed = false;
  for (let i = 0; i < 30; i++) {
    const status = await client.rpc.getSignatureStatuses([signature]).send();
    if (
      status.value[0]?.confirmationStatus === 'confirmed' ||
      status.value[0]?.confirmationStatus === 'finalized'
    ) {
      confirmed = true;
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  if (!confirmed) {
    throw new Error('Airdrop confirmation timeout');
  }

  return keypair;
}

async function airdropToAddress(
  client: SolanaClient,
  address: Address,
  lamportsAmount: bigint,
): Promise<void> {
  const signature = await client.rpc
    .requestAirdrop(address, lamports(lamportsAmount))
    .send();

  let confirmed = false;
  for (let i = 0; i < 30; i++) {
    const status = await client.rpc.getSignatureStatuses([signature]).send();
    if (
      status.value[0]?.confirmationStatus === 'confirmed' ||
      status.value[0]?.confirmationStatus === 'finalized'
    ) {
      confirmed = true;
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  if (!confirmed) {
    throw new Error('Airdrop confirmation timeout');
  }
}

/**
 * Creates a new SPL token mint.
 * @param client - The Solana client
 * @param payer - The payer/mint authority
 * @param decimals - Number of decimals for the mint
 * @returns The address of the created mint
 */
async function createMint(
  client: SolanaClient,
  payer: KeyPairSigner,
  decimals: number,
): Promise<Address> {
  const mint = await generateKeyPairSigner();
  const mintSize = getMintSize();

  const rent = await client.rpc
    .getMinimumBalanceForRentExemption(BigInt(mintSize))
    .send();

  const { value: latestBlockhash } = await client.rpc
    .getLatestBlockhash()
    .send();

  // Create account for the mint
  const createAccountIx = getCreateAccountInstruction({
    payer,
    newAccount: mint,
    lamports: rent,
    space: mintSize,
    programAddress: TOKEN_PROGRAM_ADDRESS,
  });

  // Initialize the mint with payer as mint authority
  const initializeMintIx = getInitializeMintInstruction({
    mint: mint.address,
    decimals,
    mintAuthority: payer.address,
    freezeAuthority: payer.address,
  });

  const transactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(payer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    (tx) =>
      appendTransactionMessageInstructions(
        [createAccountIx, initializeMintIx],
        tx,
      ),
  );

  const signedTransaction =
    await signTransactionMessageWithSigners(transactionMessage);
  await client.sendAndConfirmTransaction(signedTransaction);

  return mint.address;
}

/**
 * Creates an Associated Token Account and mints tokens to it.
 * @param client - The Solana client
 * @param payer - The payer (must be mint authority to mint tokens)
 * @param mint - The token mint address
 * @param owner - The owner of the ATA
 * @param amount - The amount of tokens to mint
 * @returns The address of the created ATA
 */
async function createAtaWithTokens(
  client: SolanaClient,
  payer: KeyPairSigner,
  mint: Address,
  owner: Address,
  amount: bigint,
): Promise<Address> {
  // Derive the ATA address
  const [ata] = await findAssociatedTokenPda({
    mint,
    owner,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const { value: latestBlockhash } = await client.rpc
    .getLatestBlockhash()
    .send();

  // Create ATA instruction
  const createAtaIx = await getCreateAssociatedTokenInstructionAsync({
    payer,
    mint,
    owner,
  });

  // Mint tokens instruction
  const mintToIx = getMintToInstruction({
    mint,
    token: ata,
    mintAuthority: payer,
    amount,
  });

  const transactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(payer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    (tx) => appendTransactionMessageInstructions([createAtaIx, mintToIx], tx),
  );

  const signedTransaction =
    await signTransactionMessageWithSigners(transactionMessage);
  await client.sendAndConfirmTransaction(signedTransaction);

  return ata;
}

// ============================================================================
// Backward Compatibility
// ============================================================================

/**
 * Convenience function for backward compatibility.
 * @returns A new IntegrationTest instance
 */
export async function initTestSuite(): Promise<IntegrationTest> {
  return IntegrationTest.create();
}
