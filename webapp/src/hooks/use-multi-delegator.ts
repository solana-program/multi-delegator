import { useMutation, useQueryClient } from "@tanstack/react-query";
import { address } from "gill";
import {
  findAssociatedTokenPda,
  TOKEN_2022_PROGRAM_ADDRESS,
  getCreateAssociatedTokenIdempotentInstruction,
} from "gill/programs/token";
import {
  getInitMultiDelegateInstruction,
  getCreateFixedDelegationInstruction,
  getCreateRecurringDelegationInstruction,
  getRevokeDelegationInstruction,
  getTransferFixedInstruction,
  getTransferRecurringInstruction,
  getCreatePlanInstruction,
  getUpdatePlanInstruction,
  getDeletePlanInstruction,
  getMultiDelegatePDA,
  getDelegationPDA,
  getPlanPDA,
  MAX_PLAN_DESTINATIONS,
  MAX_PLAN_PULLERS,
  ZERO_ADDRESS,
  PlanStatus,
} from "@multidelegator/client";
import { useClusterConfig } from "@/hooks/use-cluster-config";
import { useWalletUiSigner } from "../components/solana/use-wallet-ui-signer";
import { useWalletTransactionSignAndSend } from "../components/solana/use-wallet-transaction-sign-and-send";
import { useTransactionToast } from "../components/use-transaction-toast";
import { invalidateWithDelay } from "@/lib/utils";
import { getBlockTimestamp } from "@/hooks/use-time-travel";

export function useMultiDelegatorMutations() {
  const signer = useWalletUiSigner();
  const signAndSend = useWalletTransactionSignAndSend();
  const queryClient = useQueryClient();
  const toast = useTransactionToast();
  const { url: rpcUrl } = useClusterConfig();

  const initMultiDelegate = useMutation({
    mutationFn: async ({
      tokenMint,
      userAta,
      tokenProgram,
    }: {
      tokenMint: string;
      userAta: string;
      tokenProgram: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const user = signer.address;
      const [multiDelegate] = await getMultiDelegatePDA(
        user,
        address(tokenMint),
      );

      const instruction = getInitMultiDelegateInstruction({
        owner: signer,
        multiDelegate,
        tokenMint: address(tokenMint),
        userAta: address(userAta),
        tokenProgram: address(tokenProgram),
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["multiDelegate"] });
      invalidateWithDelay(queryClient, [
        ["multiDelegateStatus"],
        ["get-token-accounts"],
        ["delegations"],
      ]);
    },
    onError: (error) => toast.onError(error),
  });

  const createFixedDelegation = useMutation({
    mutationFn: async ({
      tokenMint,
      delegatee,
      nonce,
      amount,
      expiryTs,
    }: {
      tokenMint: string;
      delegatee: string;
      nonce: number | bigint;
      amount: number | bigint;
      expiryTs: number | bigint;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const user = signer.address;
      const [multiDelegate] = await getMultiDelegatePDA(
        user,
        address(tokenMint),
      );
      const [delegationAccount] = await getDelegationPDA(
        multiDelegate,
        user,
        address(delegatee),
        nonce,
      );

      const instruction = getCreateFixedDelegationInstruction({
        delegator: signer,
        multiDelegate,
        delegationAccount,
        delegatee: address(delegatee),
        fixedDelegation: { nonce, amount, expiryTs },
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["delegations"] });
    },
    onError: (error) => toast.onError(error),
  });

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
      tokenMint: string;
      delegatee: string;
      nonce: number | bigint;
      amountPerPeriod: number | bigint;
      periodLengthS: number | bigint;
      expiryTs: number | bigint;
      startTs?: number | bigint;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const user = signer.address;
      const [multiDelegate] = await getMultiDelegatePDA(
        user,
        address(tokenMint),
      );
      const [delegationAccount] = await getDelegationPDA(
        multiDelegate,
        user,
        address(delegatee),
        nonce,
      );

      const instruction = getCreateRecurringDelegationInstruction({
        delegator: signer,
        multiDelegate,
        delegationAccount,
        delegatee: address(delegatee),
        recurringDelegation: {
          nonce,
          amountPerPeriod,
          periodLengthS,
          startTs: startTs ?? await getBlockTimestamp(rpcUrl),
          expiryTs,
        },
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["delegations"] });
    },
    onError: (error) => toast.onError(error),
  });

  const revokeDelegation = useMutation({
    mutationFn: async ({
      delegationAccount,
    }: {
      delegationAccount: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const instruction = getRevokeDelegationInstruction({
        authority: signer,
        delegationAccount: address(delegationAccount),
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["delegations"] });
    },
    onError: (error) => toast.onError(error),
  });

  type TransferParams = {
    tokenMint: string;
    delegationAccount: string;
    delegator: string;
    amount: bigint;
    receiverAta?: string;
  };

  const buildTransferInstructions = async (
    params: TransferParams,
    kind: "fixed" | "recurring",
  ) => {
    if (!signer) throw new Error("Wallet not connected");

    const mint = address(params.tokenMint);
    const delegatorAddr = address(params.delegator);
    const [multiDelegate] = await getMultiDelegatePDA(delegatorAddr, mint);
    const [delegatorAta] = await findAssociatedTokenPda({
      mint,
      owner: delegatorAddr,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });
    const receiver = params.receiverAta
      ? address(params.receiverAta)
      : (
          await findAssociatedTokenPda({
            mint,
            owner: signer.address,
            tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
          })
        )[0];

    const createAtaIx = getCreateAssociatedTokenIdempotentInstruction({
      payer: signer,
      ata: receiver,
      owner: signer.address,
      mint,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });

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
    };

    const transferIx =
      kind === "fixed"
        ? getTransferFixedInstruction(transferParams)
        : getTransferRecurringInstruction(transferParams);

    return { instructions: [createAtaIx, transferIx], signer };
  };

  const transferFixed = useMutation({
    mutationFn: async (params: TransferParams) => {
      const { instructions, signer: txSigner } =
        await buildTransferInstructions(params, "fixed");
      const signature = await signAndSend(instructions, txSigner);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["delegations"] });
      invalidateWithDelay(queryClient, [["get-token-accounts"]]);
    },
    onError: (error) => toast.onError(error),
  });

  const transferRecurring = useMutation({
    mutationFn: async (params: TransferParams) => {
      const { instructions, signer: txSigner } =
        await buildTransferInstructions(params, "recurring");
      const signature = await signAndSend(instructions, txSigner);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["delegations"] });
      invalidateWithDelay(queryClient, [["get-token-accounts"]]);
    },
    onError: (error) => toast.onError(error),
  });

  const createPlan = useMutation({
    mutationFn: async ({
      planId,
      mint,
      amount,
      periodHours,
      endTs,
      destinations,
      pullers,
      metadataUri,
    }: {
      planId: bigint;
      mint: string;
      amount: bigint;
      periodHours: number;
      endTs: number;
      destinations: string[];
      pullers: string[];
      metadataUri: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const paddedDestinations = Array.from(
        { length: MAX_PLAN_DESTINATIONS },
        (_, i) => address(destinations[i] || ZERO_ADDRESS),
      );
      const paddedPullers = Array.from(
        { length: MAX_PLAN_PULLERS },
        (_, i) => address(pullers[i] || ZERO_ADDRESS),
      );

      const [planPda] = await getPlanPDA(signer.address, planId);

      const instruction = getCreatePlanInstruction({
        merchant: signer,
        planPda,
        tokenMint: address(mint),
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
        planData: {
          planId,
          mint: address(mint),
          amount,
          periodHours: BigInt(periodHours),
          endTs: BigInt(endTs),
          destinations: paddedDestinations,
          pullers: paddedPullers,
          metadataUri,
        },
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["plans"] });
    },
    onError: (error) => toast.onError(error),
  });

  const updatePlan = useMutation({
    mutationFn: async ({
      planPda,
      status,
      endTs,
      metadataUri,
    }: {
      planPda: string;
      status: PlanStatus;
      endTs: number;
      metadataUri: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const instruction = getUpdatePlanInstruction({
        owner: signer,
        planPda: address(planPda),
        updatePlanData: { status, endTs: BigInt(endTs), metadataUri },
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["plans"] });
    },
    onError: (error) => toast.onError(error),
  });

  const deletePlan = useMutation({
    mutationFn: async ({ planPda }: { planPda: string }) => {
      if (!signer) throw new Error("Wallet not connected");

      const instruction = getDeletePlanInstruction({
        owner: signer,
        planPda: address(planPda),
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["plans"] });
    },
    onError: (error) => toast.onError(error),
  });

  return {
    initMultiDelegate,
    createFixedDelegation,
    createRecurringDelegation,
    revokeDelegation,
    transferFixed,
    transferRecurring,
    createPlan,
    updatePlan,
    deletePlan,
  };
}
