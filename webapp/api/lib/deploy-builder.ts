import { getAddressFromPublicKey, createKeyPairFromBytes, generateExtractableKeyPair, extractBytesFromKeyPair } from 'gill'
import crypto from 'node:crypto'
import { CHUNK_SIZE } from './bpf-loader.js'

export { CHUNK_SIZE }

export interface DeployPlan {
  bufferKeypair: number[]
  bufferAddress: string
  programKeypair?: number[]
  chunks: string[]
  totalChunks: number
  programAddress: string
  soHash: string
  soSize: number
}

function chunkSoBytes(soBytes: Uint8Array): string[] {
  const totalChunks = Math.ceil(soBytes.length / CHUNK_SIZE)
  const chunks: string[] = []
  for (let i = 0; i < totalChunks; i++) {
    const offset = i * CHUNK_SIZE
    const chunk = soBytes.slice(offset, offset + CHUNK_SIZE)
    chunks.push(Buffer.from(chunk).toString('base64'))
  }
  return chunks
}

export async function buildDeployPlan(
  soBytes: Uint8Array,
  programKeypairBytes: Uint8Array,
): Promise<DeployPlan> {
  const soHash = crypto.createHash('sha256').update(soBytes).digest('hex')

  const bufferKp = await generateExtractableKeyPair()
  const bufferKeypairBytes = await extractBytesFromKeyPair(bufferKp)
  const bufferAddress = await getAddressFromPublicKey(bufferKp.publicKey)

  const programKp = await createKeyPairFromBytes(programKeypairBytes)
  const programAddress = await getAddressFromPublicKey(programKp.publicKey)

  const chunks = chunkSoBytes(soBytes)

  return {
    bufferKeypair: Array.from(bufferKeypairBytes),
    bufferAddress: bufferAddress.toString(),
    programKeypair: Array.from(programKeypairBytes),
    chunks,
    totalChunks: chunks.length,
    programAddress: programAddress.toString(),
    soHash,
    soSize: soBytes.length,
  }
}

export async function buildUpgradePlan(
  soBytes: Uint8Array,
  programAddress: string,
): Promise<DeployPlan> {
  const soHash = crypto.createHash('sha256').update(soBytes).digest('hex')

  const bufferKp = await generateExtractableKeyPair()
  const bufferKeypairBytes = await extractBytesFromKeyPair(bufferKp)
  const bufferAddress = await getAddressFromPublicKey(bufferKp.publicKey)

  const chunks = chunkSoBytes(soBytes)

  return {
    bufferKeypair: Array.from(bufferKeypairBytes),
    bufferAddress: bufferAddress.toString(),
    chunks,
    totalChunks: chunks.length,
    programAddress,
    soHash,
    soSize: soBytes.length,
  }
}
