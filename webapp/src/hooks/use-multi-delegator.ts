import { useMutation, useQueryClient } from '@tanstack/react-query'
import { address } from 'gill'
import {
  findAssociatedTokenPda,
  TOKEN_2022_PROGRAM_ADDRESS,
  getCreateAssociatedTokenIdempotentInstruction,
} from 'gill/programs/token'
import {
  getInitMultiDelegateInstruction,
  getCreateFixedDelegationInstruction,
  getCreateRecurringDelegationInstruction,
  getRevokeDelegationInstruction,
  getTransferFixedInstruction,
  getTransferRecurringInstruction,
  getMultiDelegatePDA,
  getDelegationPDA,
} from '@multidelegator/client'
import { useWalletUiSigner } from '../components/solana/use-wallet-ui-signer'
import { useWalletTransactionSignAndSend } from '../components/solana/use-wallet-transaction-sign-and-send'
import { useTransactionToast } from '../components/use-transaction-toast'
import { invalidateWithDelay } from '@/lib/utils'

export function useMultiDelegatorMutations() {
  const signer = useWalletUiSigner()
  const signAndSend = useWalletTransactionSignAndSend()
  const queryClient = useQueryClient()
  const toast = useTransactionToast()

  const initMultiDelegate = useMutation({
    mutationFn: async ({
      tokenMint,
      userAta,
      tokenProgram,
    }: {
      tokenMint: string
      userAta: string
      tokenProgram: string
    }) => {
      if (!signer) throw new Error('Wallet not connected')

      const user = signer.address
      const [multiDelegate] = await getMultiDelegatePDA(user, address(tokenMint))

      const instruction = getInitMultiDelegateInstruction({
        owner: signer,
        multiDelegate,
        tokenMint: address(tokenMint),
        userAta: address(userAta),
        tokenProgram: address(tokenProgram),
      })

      const signature = await signAndSend(instruction, signer)
      return { signature }
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature)
      queryClient.invalidateQueries({ queryKey: ['multiDelegate'] })
      invalidateWithDelay(queryClient, [
        ['multiDelegateStatus'],
        ['get-token-accounts'],
        ['delegations'],
      ])
    },
    onError: (error) => toast.onError(error),
  })

  const createFixedDelegation = useMutation({
    mutationFn: async ({
      tokenMint,
      delegatee,
      nonce,
      amount,
      expiryTs,
    }: {
      tokenMint: string
      delegatee: string
      nonce: number | bigint
      amount: number | bigint
      expiryTs: number | bigint
    }) => {
      if (!signer) throw new Error('Wallet not connected')

      const user = signer.address
      const [multiDelegate] = await getMultiDelegatePDA(user, address(tokenMint))
      const [delegationAccount] = await getDelegationPDA(multiDelegate, user, address(delegatee), nonce)

      const instruction = getCreateFixedDelegationInstruction({
        delegator: signer,
        multiDelegate,
        delegationAccount,
        delegatee: address(delegatee),
        nonce,
        amount,
        expiryTs,
      })

      const signature = await signAndSend(instruction, signer)
      return { signature }
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature)
      queryClient.invalidateQueries({ queryKey: ['delegations'] })
    },
    onError: (error) => toast.onError(error),
  })

  const createRecurringDelegation = useMutation({
    mutationFn: async ({
      tokenMint,
      delegatee,
      nonce,
      amountPerPeriod,
      periodLengthS,
      expiryTs,
      startTs,
    }: {
      tokenMint: string
      delegatee: string
      nonce: number | bigint
      amountPerPeriod: number | bigint
      periodLengthS: number | bigint
      expiryTs: number | bigint
      startTs?: number | bigint
    }) => {
      if (!signer) throw new Error('Wallet not connected')

      const user = signer.address
      const [multiDelegate] = await getMultiDelegatePDA(user, address(tokenMint))
      const [delegationAccount] = await getDelegationPDA(multiDelegate, user, address(delegatee), nonce)

      const instruction = getCreateRecurringDelegationInstruction({
        delegator: signer,
        multiDelegate,
        delegationAccount,
        delegatee: address(delegatee),
        nonce,
        amountPerPeriod,
        periodLengthS,
        startTs: startTs ?? Math.floor(Date.now() / 1000),
        expiryTs,
      })

      const signature = await signAndSend(instruction, signer)
      return { signature }
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature)
      queryClient.invalidateQueries({ queryKey: ['delegations'] })
    },
    onError: (error) => toast.onError(error),
  })

  const revokeDelegation = useMutation({
    mutationFn: async ({ delegationAccount }: { delegationAccount: string }) => {
      if (!signer) throw new Error('Wallet not connected')

      const instruction = getRevokeDelegationInstruction({
        authority: signer,
        delegationAccount: address(delegationAccount),
      })

      const signature = await signAndSend(instruction, signer)
      return { signature }
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature)
      queryClient.invalidateQueries({ queryKey: ['delegations'] })
    },
    onError: (error) => toast.onError(error),
  })

  type TransferParams = {
    tokenMint: string
    delegationAccount: string
    delegator: string
    amount: bigint
    receiverAta?: string
  }

  const buildTransferInstructions = async (
    params: TransferParams,
    kind: 'fixed' | 'recurring'
  ) => {
    if (!signer) throw new Error('Wallet not connected')

    const mint = address(params.tokenMint)
    const delegatorAddr = address(params.delegator)
    const [multiDelegate] = await getMultiDelegatePDA(delegatorAddr, mint)
    const [delegatorAta] = await findAssociatedTokenPda({ mint, owner: delegatorAddr, tokenProgram: TOKEN_2022_PROGRAM_ADDRESS })
    const receiver = params.receiverAta
      ? address(params.receiverAta)
      : (await findAssociatedTokenPda({ mint, owner: signer.address, tokenProgram: TOKEN_2022_PROGRAM_ADDRESS }))[0]

    const createAtaIx = getCreateAssociatedTokenIdempotentInstruction({
      payer: signer,
      ata: receiver,
      owner: signer.address,
      mint,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    })

    const transferParams = {
      delegationPda: address(params.delegationAccount),
      multiDelegate,
      delegatorAta,
      receiverAta: receiver,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
      delegatee: signer,
      transferData: {
        amount: params.amount,
        delegator: delegatorAddr,
        mint,
      },
    }

    const transferIx = kind === 'fixed'
      ? getTransferFixedInstruction(transferParams)
      : getTransferRecurringInstruction(transferParams)

    return { instructions: [createAtaIx, transferIx], signer }
  }

  const transferFixed = useMutation({
    mutationFn: async (params: TransferParams) => {
      const { instructions, signer: txSigner } = await buildTransferInstructions(params, 'fixed')
      const signature = await signAndSend(instructions, txSigner)
      return { signature }
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature)
      queryClient.invalidateQueries({ queryKey: ['delegations'] })
      invalidateWithDelay(queryClient, [['get-token-accounts']])
    },
    onError: (error) => toast.onError(error),
  })

  const transferRecurring = useMutation({
    mutationFn: async (params: TransferParams) => {
      const { instructions, signer: txSigner } = await buildTransferInstructions(params, 'recurring')
      const signature = await signAndSend(instructions, txSigner)
      return { signature }
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature)
      queryClient.invalidateQueries({ queryKey: ['delegations'] })
      invalidateWithDelay(queryClient, [['get-token-accounts']])
    },
    onError: (error) => toast.onError(error),
  })

  return {
    initMultiDelegate,
    createFixedDelegation,
    createRecurringDelegation,
    revokeDelegation,
    transferFixed,
    transferRecurring,
  }
}
