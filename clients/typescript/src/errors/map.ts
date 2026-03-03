import { ProgramError } from './types.js';

const CUSTOM_ERROR_REGEX = /custom program error: 0x([0-9a-fA-F]+)/;

function extractMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  if (typeof error === 'object' && error !== null && 'message' in error)
    return String((error as { message: unknown }).message);
  return '';
}

function extractLogs(error: unknown): string[] {
  if (typeof error !== 'object' || error === null) return [];
  if ('logs' in error && Array.isArray((error as { logs: unknown }).logs))
    return (error as { logs: string[] }).logs;
  return [];
}

/**
 * Extracts a custom program error code from a transaction failure.
 * Searches the error message and transaction logs for hex error codes.
 *
 * @param error - The caught error (Error instance, string, object with message/logs, or unknown).
 * @returns A {@link ProgramError} when a hex error code is found, otherwise `null`.
 */
export function parseProgramError(error: unknown): ProgramError | null {
  if (error == null) return null;

  const message = extractMessage(error);
  const match = CUSTOM_ERROR_REGEX.exec(message);
  if (match?.[1]) {
    return new ProgramError(Number.parseInt(match[1], 16));
  }

  for (const log of extractLogs(error)) {
    const logMatch = CUSTOM_ERROR_REGEX.exec(log);
    if (logMatch?.[1]) {
      return new ProgramError(Number.parseInt(logMatch[1], 16));
    }
  }

  return null;
}
