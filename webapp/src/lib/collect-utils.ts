import type { PlanSubscriber } from '@/hooks/use-subscriptions'

export interface EligibleSubscriber {
  subscriptionAddress: string
  delegator: string
  collectAmount: bigint
}

export function computeEligibleSubscribers(
  subscribers: PlanSubscriber[],
  planAmount: bigint,
  periodHours: bigint,
  currentTs: number,
): EligibleSubscriber[] {
  if (planAmount <= 0n || periodHours <= 0n) return []

  const eligible: EligibleSubscriber[] = []

  for (const sub of subscribers) {
    if (sub.expiresAtTs !== 0n && currentTs >= Number(sub.expiresAtTs)) continue

    const periodEnd = Number(sub.currentPeriodStartTs) + Number(periodHours) * 3600
    const collectAmount = currentTs >= periodEnd
      ? planAmount
      : planAmount - sub.amountPulledInPeriod

    if (collectAmount <= 0n) continue

    eligible.push({
      subscriptionAddress: sub.subscriptionAddress,
      delegator: sub.delegator,
      collectAmount,
    })
  }

  return eligible
}
