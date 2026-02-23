import { readFile, writeFile } from 'fs/promises'
import { existsSync } from 'fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const CONFIG_PATH = join(__dirname, '../config.json')

export interface TokenConfig {
  symbol: string
  name: string
  mint: string
  decimals: number
  type: 'test' | 'mainnet'
  description: string
}

export interface Config {
  network: string
  adminWallet: string
  tokens: TokenConfig[]
}

export async function readConfig(): Promise<Config> {
  if (!existsSync(CONFIG_PATH)) {
    return {
      network: 'localnet',
      adminWallet: '~/.config/solana/id.json',
      tokens: [],
    }
  }
  const content = await readFile(CONFIG_PATH, 'utf-8')
  return JSON.parse(content)
}

export async function writeConfig(config: Config): Promise<void> {
  await writeFile(CONFIG_PATH, JSON.stringify(config, null, 2))
}

export async function addToken(token: TokenConfig): Promise<void> {
  const config = await readConfig()
  const existingIndex = config.tokens.findIndex((t) => t.symbol === token.symbol)
  if (existingIndex >= 0) {
    config.tokens[existingIndex] = token
  } else {
    config.tokens.push(token)
  }
  await writeConfig(config)
}

export async function updateNetwork(network: string): Promise<void> {
  const config = await readConfig()
  config.network = network
  await writeConfig(config)
}

export async function clearConfig(): Promise<void> {
  await writeConfig({
    network: 'localnet',
    adminWallet: '~/.config/solana/id.json',
    tokens: [],
  })
}

export async function getUsdcMint(): Promise<string | null> {
  const config = await readConfig()
  const usdc = config.tokens.find((t) => t.symbol === 'USDC')
  return usdc?.mint ?? null
}
