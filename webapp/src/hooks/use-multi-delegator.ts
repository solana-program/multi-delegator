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
  getSubscribeInstruction,
  getCancelSubscriptionInstruction,
  getTransferSubscriptionInstruction,
  getMultiDelegatePDA,
  getDelegationPDA,
  getPlanPDA,
  getSubscriptionPDA,
  getEventAuthorityPDA,
  MULTI_DELEGATOR_PROGRAM_ADDRESS,
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
import { packInstructionBatches } from "@/lib/tx-packer";
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

    const [eventAuthority] = await getEventAuthorityPDA();

    const transferParams = {
      delegationPda: address(params.delegationAccount),
      multiDelegate,
      delegatorAta,
      receiverAta: receiver,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
      delegatee: signer,
      eventAuthority,
      selfProgram: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
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

  const subscribe = useMutation({
    mutationFn: async ({
      merchant,
      planId,
      tokenMint,
    }: {
      merchant: string;
      planId: bigint;
      tokenMint: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const [planPda, planBump] = await getPlanPDA(address(merchant), planId);
      const [subscriptionPda] = await getSubscriptionPDA(planPda, signer.address);
      const [multiDelegatePda] = await getMultiDelegatePDA(signer.address, address(tokenMint));
      const [eventAuthority] = await getEventAuthorityPDA();

      const instruction = getSubscribeInstruction({
        subscriber: signer,
        merchant: address(merchant),
        planPda,
        subscriptionPda,
        multiDelegatePda,
        eventAuthority,
        selfProgram: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
        subscribeData: { planId, planBump },
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
    },
    onError: (error) => toast.onError(error),
  });

  const cancelSubscription = useMutation({
    mutationFn: async ({
      planPda,
      subscriptionPda,
    }: {
      planPda: string;
      subscriptionPda: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const [eventAuthority] = await getEventAuthorityPDA();

      const instruction = getCancelSubscriptionInstruction({
        subscriber: signer,
        planPda: address(planPda),
        subscriptionPda: address(subscriptionPda),
        eventAuthority,
        selfProgram: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
    },
    onError: (error) => toast.onError(error),
  });

  const revokeSubscription = useMutation({
    mutationFn: async ({
      subscriptionPda,
    }: {
      subscriptionPda: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const instruction = getRevokeDelegationInstruction({
        authority: signer,
        delegationAccount: address(subscriptionPda),
      });

      const signature = await signAndSend(instruction, signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
    },
    onError: (error) => toast.onError(error),
  });

  const cancelAndRevokeSubscription = useMutation({
    mutationFn: async ({
      planPda,
      subscriptionPda,
    }: {
      planPda: string;
      subscriptionPda: string;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const [eventAuthority] = await getEventAuthorityPDA();

      const cancelIx = getCancelSubscriptionInstruction({
        subscriber: signer,
        planPda: address(planPda),
        subscriptionPda: address(subscriptionPda),
        eventAuthority,
        selfProgram: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
      });

      const revokeIx = getRevokeDelegationInstruction({
        authority: signer,
        delegationAccount: address(subscriptionPda),
      });

      const signature = await signAndSend([cancelIx, revokeIx], signer);
      return { signature };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signature);
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
    },
    onError: (error) => toast.onError(error),
  });

  const collectSubscriptionPayments = useMutation({
    mutationFn: async ({
      planAddress,
      subscribers,
      mint,
      destinations,
    }: {
      planAddress: string;
      subscribers: Array<{ subscriptionAddress: string; delegator: string; amount: bigint }>;
      mint: string;
      destinations: string[];
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      const mintAddr = address(mint);
      const planPda = address(planAddress);

      const firstDest = destinations.find((d) => d !== ZERO_ADDRESS);
      const receiverOwner = firstDest ? address(firstDest) : signer.address;
      const [receiverAta] = await findAssociatedTokenPda({
        mint: mintAddr,
        owner: receiverOwner,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
      });

      const createAtaIx = getCreateAssociatedTokenIdempotentInstruction({
        payer: signer,
        ata: receiverAta,
        owner: receiverOwner,
        mint: mintAddr,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
      });

      const [eventAuthority] = await getEventAuthorityPDA();

      const transferIxs = await Promise.all(
        subscribers.map(async (sub) => {
          const delegatorAddr = address(sub.delegator);
          const [multiDelegate] = await getMultiDelegatePDA(delegatorAddr, mintAddr);
          const [delegatorAta] = await findAssociatedTokenPda({
            mint: mintAddr,
            owner: delegatorAddr,
            tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
          });

          return getTransferSubscriptionInstruction({
            subscriptionPda: address(sub.subscriptionAddress),
            planPda,
            multiDelegate,
            delegatorAta,
            receiverAta,
            caller: signer,
            tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
            eventAuthority,
            selfProgram: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
            transferData: {
              amount: sub.amount,
              delegator: delegatorAddr,
              mint: mintAddr,
            },
          });
        })
      );

      const signatures: string[] = [];
      let collected = 0;
      const batches = packInstructionBatches(transferIxs, signer, [createAtaIx]);

      for (let b = 0; b < batches.length; b++) {
        try {
          signatures.push(await signAndSend(batches[b], signer));
          collected += batches[b].length - (b === 0 ? 1 : 0);
        } catch (err) {
          if (collected === 0) throw err;
          console.warn(
            `Batch failed after collecting ${collected}/${subscribers.length}:`,
            err instanceof Error ? err.message : err,
          );
          return { signatures, collected, partial: true };
        }
      }

      return { signatures, collected, partial: false };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signatures[0]);
      invalidateWithDelay(queryClient, [
        ["subscriberCounts"],
        ["get-token-accounts"],
      ]);
    },
    onError: (error) => toast.onError(error),
  });

  const collectAllPlanPayments = useMutation({
    mutationFn: async ({
      plans,
    }: {
      plans: Array<{
        planAddress: string;
        subscribers: Array<{ subscriptionAddress: string; delegator: string; amount: bigint }>;
        mint: string;
        destinations: string[];
      }>;
    }) => {
      if (!signer) throw new Error("Wallet not connected");

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const ataIxs: any[] = [];
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const transferIxs: any[] = [];
      const seenAtas = new Set<string>();
      const [eventAuthority] = await getEventAuthorityPDA();

      for (const plan of plans) {
        const mintAddr = address(plan.mint);
        const planPda = address(plan.planAddress);
        const firstDest = plan.destinations.find((d) => d !== ZERO_ADDRESS);
        const receiverOwner = firstDest ? address(firstDest) : signer.address;
        const [receiverAta] = await findAssociatedTokenPda({
          mint: mintAddr,
          owner: receiverOwner,
          tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
        });

        const ataKey = receiverAta.toString();
        if (!seenAtas.has(ataKey)) {
          seenAtas.add(ataKey);
          ataIxs.push(
            getCreateAssociatedTokenIdempotentInstruction({
              payer: signer,
              ata: receiverAta,
              owner: receiverOwner,
              mint: mintAddr,
              tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
            }),
          );
        }

        for (const sub of plan.subscribers) {
          const delegatorAddr = address(sub.delegator);
          const [multiDelegate] = await getMultiDelegatePDA(delegatorAddr, mintAddr);
          const [delegatorAta] = await findAssociatedTokenPda({
            mint: mintAddr,
            owner: delegatorAddr,
            tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
          });

          transferIxs.push(
            getTransferSubscriptionInstruction({
              subscriptionPda: address(sub.subscriptionAddress),
              planPda,
              multiDelegate,
              delegatorAta,
              receiverAta,
              caller: signer,
              tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
              eventAuthority,
              selfProgram: address(MULTI_DELEGATOR_PROGRAM_ADDRESS),
              transferData: {
                amount: sub.amount,
                delegator: delegatorAddr,
                mint: mintAddr,
              },
            }),
          );
        }
      }

      const signatures: string[] = [];
      let collected = 0;
      const batches = packInstructionBatches(transferIxs, signer, ataIxs);

      for (let b = 0; b < batches.length; b++) {
        try {
          signatures.push(await signAndSend(batches[b], signer));
          collected += batches[b].length - (b === 0 ? ataIxs.length : 0);
        } catch (err) {
          if (collected === 0) throw err;
          console.warn(
            `Batch failed after collecting ${collected}/${transferIxs.length}:`,
            err instanceof Error ? err.message : err,
          );
          return { signatures, collected, total: transferIxs.length, partial: true };
        }
      }

      return { signatures, collected, total: transferIxs.length, partial: false };
    },
    onSuccess: (res) => {
      toast.onSuccess(res.signatures[0]);
      invalidateWithDelay(queryClient, [
        ["subscriberCounts"],
        ["get-token-accounts"],
      ]);
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
    subscribe,
    cancelSubscription,
    revokeSubscription,
    cancelAndRevokeSubscription,
    collectSubscriptionPayments,
    collectAllPlanPayments,
  };
}
