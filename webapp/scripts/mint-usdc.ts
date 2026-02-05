#!/usr/bin/env bun
/**
 * Mints mock USDC to a recipient wallet
 * Usage: bun scripts/mint-usdc.ts <recipient-address> <amount>
 */

import { address } from 'gill'
import { getUsdcMint } from './config-manager'
import { mintMockUsdc } from './helpers'

async function main() {
  const args = process.argv.slice(2)

  if (args.length < 2) {
    console.error('Usage: bun scripts/mint-usdc.ts <recipient-address> <amount>')
    console.error('Example: bun scripts/mint-usdc.ts 7xyz... 1000')
    process.exit(1)
  }

  const [recipientAddress, amountStr] = args
  const amount = parseFloat(amountStr)

  if (isNaN(amount) || amount <= 0) {
    console.error('Amount must be a positive number')
    process.exit(1)
  }

  try {
    // Get USDC mint from config
    const usdcMint = await getUsdcMint()
    if (!usdcMint) {
      console.error('USDC mint not configured. Run init-test-environment.ts first.')
      process.exit(1)
    }

    console.log(`Minting ${amount} USDC to ${recipientAddress}...`)

    // Convert to smallest units (6 decimals)
    const amountInSmallestUnits = BigInt(Math.round(amount * 1_000_000))

    await mintMockUsdc(
      address(usdcMint),
      address(recipientAddress),
      amountInSmallestUnits,
    )

    console.log(`Successfully minted ${amount} USDC to ${recipientAddress}`)
  } catch (error) {
    console.error('Failed to mint USDC:', error)
    process.exit(1)
  }
}

main()
