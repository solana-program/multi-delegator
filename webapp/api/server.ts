import { spawn } from 'child_process'
import { createServer } from 'node:http'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { readFile } from 'fs/promises'

const __dirname = dirname(fileURLToPath(import.meta.url))

const PORT = 3001
const RPC_URL = 'http://127.0.0.1:8899'
const CONFIG_PATH = join(__dirname, '../config.json')

const MIN_SOL_AIRDROP = 0.1
const MAX_SOL_AIRDROP = 10

interface Config {
  network: string
  tokens: Array<{ symbol: string; mint: string; decimals: number }>
}

function log(level: 'info' | 'warn' | 'error', message: string, meta?: Record<string, unknown>) {
  const timestamp = new Date().toISOString()
  const metaStr = meta ? ` ${JSON.stringify(meta)}` : ''
  console.log(`[${timestamp}] ${level.toUpperCase()}: ${message}${metaStr}`)
}

async function readConfig(): Promise<Config> {
  try {
    const content = await readFile(CONFIG_PATH, 'utf-8')
    return JSON.parse(content)
  } catch (error) {
    log('warn', 'Failed to read config, using defaults', { error: String(error), path: CONFIG_PATH })
    return { network: 'localnet', tokens: [] }
  }
}

function corsHeaders() {
  return {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type',
  }
}

function jsonResponse(data: unknown, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json', ...corsHeaders() },
  })
}

async function handleSolAirdrop(recipient: string, amount: number): Promise<Response> {
  if (!recipient || !amount || amount < MIN_SOL_AIRDROP || amount > MAX_SOL_AIRDROP) {
    return jsonResponse({ error: `Invalid parameters. Amount must be between ${MIN_SOL_AIRDROP} and ${MAX_SOL_AIRDROP}` }, 400)
  }

  return new Promise((resolve) => {
    const child = spawn('solana', ['airdrop', String(amount), recipient, '--url', RPC_URL])

    let stdout = ''
    let stderr = ''

    child.stdout.on('data', (data) => (stdout += data.toString()))
    child.stderr.on('data', (data) => (stderr += data.toString()))

    child.on('close', (code) => {
      if (code !== 0) {
        resolve(jsonResponse({ error: 'Failed to airdrop SOL', details: stderr }, 500))
        return
      }
      resolve(
        jsonResponse({
          success: true,
          message: `Successfully airdropped ${amount} SOL to ${recipient}`,
          recipient,
          amount,
        }),
      )
    })

    child.on('error', (err) => {
      resolve(jsonResponse({ error: 'Failed to start airdrop process', details: err.message }, 500))
    })
  })
}

async function handleUsdcAirdrop(recipient: string, amount: number): Promise<Response> {
  if (!recipient || !amount || amount <= 0) {
    return jsonResponse({ error: 'Invalid parameters' }, 400)
  }

  const config = await readConfig()
  const usdcMint = config.tokens.find((t) => t.symbol === 'USDC')?.mint
  if (!usdcMint) {
    return jsonResponse({ error: 'USDC mint not configured. Run init-test-environment.ts first.' }, 500)
  }

  return new Promise((resolve) => {
    const scriptPath = join(__dirname, '../scripts/mint-usdc.ts')
    const child = spawn('tsx', [scriptPath, recipient, String(amount)])

    let stdout = ''
    let stderr = ''

    child.stdout.on('data', (data) => (stdout += data.toString()))
    child.stderr.on('data', (data) => (stderr += data.toString()))

    child.on('close', (code) => {
      if (code !== 0) {
        resolve(jsonResponse({ error: 'Failed to mint USDC', details: stderr || stdout }, 500))
        return
      }
      resolve(
        jsonResponse({
          success: true,
          message: `Successfully minted ${amount} USDC to ${recipient}`,
          recipient,
          amount,
          mint: usdcMint,
        }),
      )
    })

    child.on('error', (err) => {
      resolve(jsonResponse({ error: 'Failed to start minting process', details: err.message }, 500))
    })
  })
}

async function parseJsonBody(req: Request): Promise<{ success: true; data: unknown } | { success: false; error: string }> {
  try {
    const data = await req.json()
    return { success: true, data }
  } catch {
    return { success: false, error: 'Invalid JSON body' }
  }
}

async function handleRequest(req: Request): Promise<Response> {
  const url = new URL(req.url)
  const startTime = Date.now()

  if (req.method === 'OPTIONS') {
    return new Response(null, { headers: corsHeaders() })
  }

  let response: Response

  if (url.pathname === '/api/health') {
    response = jsonResponse({ status: 'ok' })
  } else if (url.pathname === '/api/config' && req.method === 'GET') {
    const config = await readConfig()
    response = jsonResponse(config)
  } else if (url.pathname === '/api/tokens' && req.method === 'GET') {
    const config = await readConfig()
    response = jsonResponse(config.tokens)
  } else if (url.pathname === '/api/airdrop/sol' && req.method === 'POST') {
    const parseResult = await parseJsonBody(req)
    if (!parseResult.success) {
      response = jsonResponse({ error: parseResult.error }, 400)
    } else {
      const body = parseResult.data as { recipient?: string; amount?: number }
      response = await handleSolAirdrop(body.recipient ?? '', body.amount ?? 0)
    }
  } else if (url.pathname === '/api/airdrop/usdc' && req.method === 'POST') {
    const parseResult = await parseJsonBody(req)
    if (!parseResult.success) {
      response = jsonResponse({ error: parseResult.error }, 400)
    } else {
      const body = parseResult.data as { recipient?: string; amount?: number }
      response = await handleUsdcAirdrop(body.recipient ?? '', body.amount ?? 0)
    }
  } else {
    response = jsonResponse({ error: 'Not found' }, 404)
  }

  const duration = Date.now() - startTime
  log('info', `${req.method} ${url.pathname}`, { status: response.status, duration: `${duration}ms` })

  return response
}

const server = createServer(async (req, res) => {
  const url = `http://localhost:${PORT}${req.url}`
  const headers = new Headers()
  for (const [key, value] of Object.entries(req.headers)) {
    if (value) headers.set(key, Array.isArray(value) ? value.join(', ') : value)
  }

  let body: string | undefined
  if (req.method === 'POST' || req.method === 'PUT') {
    body = await new Promise<string>((resolve) => {
      let data = ''
      req.on('data', (chunk) => (data += chunk))
      req.on('end', () => resolve(data))
    })
  }

  const request = new Request(url, { method: req.method ?? 'GET', headers, body })
  const response = await handleRequest(request)

  const respHeaders: Record<string, string> = {}
  response.headers.forEach((v, k) => { respHeaders[k] = v })
  res.writeHead(response.status, respHeaders)
  res.end(await response.text())
})

server.listen(PORT, () => {
  console.log(`Multi-Delegator API server running on port ${PORT}`)
  console.log('')
  console.log('Endpoints:')
  console.log(`  GET  http://localhost:${PORT}/api/health`)
  console.log(`  GET  http://localhost:${PORT}/api/config`)
  console.log(`  GET  http://localhost:${PORT}/api/tokens`)
  console.log(`  POST http://localhost:${PORT}/api/airdrop/sol`)
  console.log(`  POST http://localhost:${PORT}/api/airdrop/usdc`)
})
