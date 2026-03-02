import { useMutation } from '@tanstack/react-query'
import {
  createTransaction,
  signAndSendTransactionMessageWithSigners,
  generateExtractableKeyPairSigner,
  type TransactionSendingSigner,
  type Address,
} from 'gill'
import {
  TOKEN_2022_PROGRAM_ADDRESS,
  getInitializeMint2Instruction,
  getMintSize,
  getMintToInstruction,
  findAssociatedTokenPda,
  getCreateAssociatedTokenIdempotentInstruction,
} from 'gill/programs/token'
import { useWalletUiSigner } from '@/components/solana/use-wallet-ui-signer'
import { useRpc } from '@/hooks/use-rpc'
import { useTransactionToast } from '@/components/use-transaction-toast'
import { buildCreateAccountIx, SYSTEM_PROGRAM } from '@/lib/bpf-loader-browser'

export function useCreateToken() {
  const walletSigner = useWalletUiSigner()
  const toast = useTransactionToast()
  const rpc = useRpc()

  const createToken = useMutation({
    mutationFn: async ({ decimals = 6 }: { decimals?: number } = {}) => {
      if (!walletSigner) throw new Error('Wallet not connected')
      const signer = walletSigner as TransactionSendingSigner

      const mintKp = await generateExtractableKeyPairSigner()
      const mintSize = getMintSize()
      const rentLamports = await rpc.getMinimumBalanceForRentExemption(BigInt(mintSize)).send()
      const { value: latestBlockhash } = await rpc.getLatestBlockhash().send()

      const createAccIx = buildCreateAccountIx(
        signer,
        mintKp,
        rentLamports,
        mintSize,
        TOKEN_2022_PROGRAM_ADDRESS as Address,
      )

      const initMintIx = getInitializeMint2Instruction({
        mint: mintKp.address,
        decimals,
        mintAuthority: signer.address,
        freezeAuthority: signer.address,
      })

      const tx = createTransaction({
        feePayer: signer,
        version: 0,
        latestBlockhash,
        instructions: [createAccIx, initMintIx],
      })

      await signAndSendTransactionMessageWithSigners(tx)

      return { mint: mintKp.address as Address }
    },
    onSuccess: () => { toast.onSuccess('Token mint created') },
    onError: (e) => toast.onError(e),
  })

  const mintTo = useMutation({
    mutationFn: async ({ mint, amount, recipient }: { mint: Address; amount: bigint; recipient?: Address }) => {
      if (!walletSigner) throw new Error('Wallet not connected')
      const signer = walletSigner as TransactionSendingSigner
      const owner = recipient ?? signer.address

      const [ata] = await findAssociatedTokenPda({
        owner,
        mint,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS as Address,
      })

      const { value: latestBlockhash } = await rpc.getLatestBlockhash().send()

      const createAtaIx = getCreateAssociatedTokenIdempotentInstruction({
        payer: signer,
        owner,
        ata,
        mint,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS as Address,
        systemProgram: SYSTEM_PROGRAM as Address,
      })

      const mintToIx = getMintToInstruction({
        mint,
        token: ata,
        mintAuthority: signer,
        amount,
      })

      const tx = createTransaction({
        feePayer: signer,
        version: 0,
        latestBlockhash,
        instructions: [createAtaIx, mintToIx],
      })

      await signAndSendTransactionMessageWithSigners(tx)
      return { ata }
    },
    onSuccess: () => { toast.onSuccess('Tokens minted') },
    onError: (e) => toast.onError(e),
  })

  return { createToken, mintTo }
}
