import { cva } from 'class-variance-authority'
import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'
import type { QueryClient } from '@tanstack/react-query'

export const USDC_DECIMALS = 6
export const USDC_MULTIPLIER = 10 ** USDC_DECIMALS
export const SECONDS_PER_DAY = 86400

export function isExpired(expiryTs: bigint): boolean {
  return Number(expiryTs) < Math.floor(Date.now() / 1000)
}

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function decodeBase64ToUint8Array(base64: string): Uint8Array {
  const binaryString = atob(base64)
  const bytes = new Uint8Array(binaryString.length)
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i)
  }
  return bytes
}

export function ellipsify(str = '', len = 4, delimiter = '..') {
  const strLen = str.length
  const limit = len * 2 + delimiter.length

  return strLen >= limit ? str.substring(0, len) + delimiter + str.substring(strLen - len, strLen) : str
}

export function toDecimals(amount: bigint, decimals: number): string {
  return (Number(amount) / 10 ** decimals).toFixed(decimals)
}

export type InvalidateQueryKeys = readonly (readonly string[])[]

export function invalidateWithDelay(
  queryClient: QueryClient,
  queryKeys: InvalidateQueryKeys,
  delayMs = 500
): void {
  queryKeys.forEach((key) => queryClient.invalidateQueries({ queryKey: [...key] }))
  setTimeout(() => {
    queryKeys.forEach((key) => void queryClient.invalidateQueries({ queryKey: [...key] }))
  }, delayMs)
}

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground shadow-xs hover:bg-primary/90',
        destructive:
          'bg-destructive text-white shadow-xs hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60',
        outline:
          'border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50',
        secondary: 'bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80',
        ghost: 'hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50',
        link: 'text-primary underline-offset-4 hover:underline',
      },
      size: {
        default: 'h-9 px-4 py-2 has-[>svg]:px-3',
        sm: 'h-8 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5',
        lg: 'h-10 rounded-md px-6 has-[>svg]:px-4',
        icon: 'size-9',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)
