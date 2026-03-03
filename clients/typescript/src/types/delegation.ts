import type { Address } from 'gill';
import type {
  FixedDelegation,
  RecurringDelegation,
  SubscriptionDelegation,
} from '../generated/index.js';

/** Discriminated union pairing each delegation variant with its on-chain address. */
export type Delegation =
  | { kind: 'fixed'; address: Address; data: FixedDelegation }
  | { kind: 'recurring'; address: Address; data: RecurringDelegation }
  | {
      kind: 'subscription';
      address: Address;
      data: SubscriptionDelegation;
    };

/**
 * Narrows a {@link Delegation} to the `fixed` variant.
 *
 * @param d - The delegation to check.
 * @returns `true` if `d` is a fixed delegation.
 */
export function isFixedDelegation(
  d: Delegation,
): d is Delegation & { kind: 'fixed' } {
  return d.kind === 'fixed';
}

/**
 * Narrows a {@link Delegation} to the `recurring` variant.
 *
 * @param d - The delegation to check.
 * @returns `true` if `d` is a recurring delegation.
 */
export function isRecurringDelegation(
  d: Delegation,
): d is Delegation & { kind: 'recurring' } {
  return d.kind === 'recurring';
}

/**
 * Narrows a {@link Delegation} to the `subscription` variant.
 *
 * @param d - The delegation to check.
 * @returns `true` if `d` is a subscription delegation.
 */
export function isSubscriptionDelegation(
  d: Delegation,
): d is Delegation & { kind: 'subscription' } {
  return d.kind === 'subscription';
}
