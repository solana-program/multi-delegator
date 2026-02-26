import {
  createTransaction,
  compileTransaction,
  getBase64EncodedWireTransaction,
  type Instruction,
  type TransactionSendingSigner,
} from 'gill'

const MAX_TX_BYTES = 1232

function base64ByteLength(b64: string): number {
  const padding = b64.endsWith('==') ? 2 : b64.endsWith('=') ? 1 : 0
  return (b64.length * 3) / 4 - padding
}

function txByteSize(instructions: Instruction[], feePayer: TransactionSendingSigner): number {
  try {
    const tx = createTransaction({
      feePayer,
      version: 0,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      latestBlockhash: { blockhash: '11111111111111111111111111111111' as any, lastValidBlockHeight: 0n },
      instructions,
    })
    return base64ByteLength(getBase64EncodedWireTransaction(compileTransaction(tx)))
  } catch {
    return MAX_TX_BYTES + 1
  }
}

export function packInstructionBatches(
  ixs: Instruction[],
  feePayer: TransactionSendingSigner,
  prefixIxs: Instruction[] = [],
): Instruction[][] {
  if (ixs.length === 0) return prefixIxs.length > 0 ? [prefixIxs] : []

  const batches: Instruction[][] = []
  let cursor = 0
  let isFirst = true

  while (cursor < ixs.length) {
    const prefix = isFirst ? prefixIxs : []
    let count = 0

    for (let i = cursor; i < ixs.length; i++) {
      const candidate = [...prefix, ...ixs.slice(cursor, i + 1)]
      if (txByteSize(candidate, feePayer) > MAX_TX_BYTES) break
      count = i - cursor + 1
    }

    if (count === 0) {
      batches.push([...prefix, ixs[cursor]])
      cursor++
    } else {
      batches.push([...prefix, ...ixs.slice(cursor, cursor + count)])
      cursor += count
    }
    isFirst = false
  }

  return batches
}
