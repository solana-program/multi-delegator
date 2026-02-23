/**
 * Initializes test environment for webapp development
 * Idempotent - checks on-chain state before creating resources
 */

import { createSolanaRpc, address } from 'gill'
import { readConfig, addToken, clearConfig, updateNetwork } from './config-manager'
import { createMockUsdc } from './helpers'

const RPC_URL = 'http://127.0.0.1:8899'

async function main() {
  console.log('Initializing test environment...')

  try {
    const rpc = createSolanaRpc(RPC_URL)

    // Wait for validator
    console.log('Waiting for validator...')
    await rpc.getLatestBlockhash().send()
    await new Promise((r) => setTimeout(r, 1000))
    console.log('Validator ready')

    // Read existing config
    const existingConfig = await readConfig()

    // Check if we have existing USDC mint on-chain
    let usdcMintString = existingConfig.tokens?.[0]?.mint
    let needsNewMint = true

    if (usdcMintString) {
      try {
        const accountInfo = await rpc.getAccountInfo(address(usdcMintString)).send()
        if (accountInfo.value) {
          console.log('Existing mock USDC mint found on-chain:', usdcMintString)
          needsNewMint = false
        }
      } catch {
        console.log('Configured USDC mint not found on-chain, will create new one')
      }
    }

    if (needsNewMint) {
      console.log('Creating new mock USDC mint...')
      usdcMintString = await createMockUsdc()
      console.log('New mock USDC created:', usdcMintString)
    }

    // Update config with validated addresses
    console.log('Updating configuration...')
    await clearConfig()
    await updateNetwork('localnet')

    await addToken({
      symbol: 'USDC',
      name: 'Mock USDC',
      mint: usdcMintString!,
      decimals: 6,
      type: 'test',
      description: 'Mock USDC for local development',
    })

    console.log('Test environment ready!')
    console.log('')
    console.log('USDC Mint:', usdcMintString)
  } catch (error) {
    console.error('Failed to initialize:', error)
    process.exit(1)
  }
}

main()
