import idl from '@idl'

type IdlError = { code: number; name: string; msg: string }

const ERROR_MESSAGES: Record<number, string> = Object.fromEntries(
  (idl.errors as IdlError[]).map(e => [e.code, e.msg])
)

export function parseProgramError(error: unknown): string {
  if (!(error instanceof Error)) return 'Unknown error'

  const hexMatch = error.message.match(/custom program error: 0x([0-9a-fA-F]+)/i)
  const decMatch = error.message.match(/Custom\((\d+)\)/)

  const code = hexMatch
    ? parseInt(hexMatch[1], 16)
    : decMatch
    ? parseInt(decMatch[1], 10)
    : null

  if (code === null) return error.message

  return ERROR_MESSAGES[code] ?? `Unknown error code: ${code}`
}
