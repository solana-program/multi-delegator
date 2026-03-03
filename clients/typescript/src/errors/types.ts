import {
  getMultiDelegatorErrorMessage,
  type MultiDelegatorError,
} from '../generated/index.js';

/** Base error class for the MultiDelegator SDK, carrying a machine-readable `code`. */
export class MultiDelegatorSDKError extends Error {
  readonly code: string;

  constructor(message: string, code: string) {
    super(message);
    this.name = 'MultiDelegatorSDKError';
    this.code = code;
  }
}

/** Wraps an on-chain program error code, resolving it to a human-readable message. */
export class ProgramError extends MultiDelegatorSDKError {
  readonly errorCode: number;

  constructor(errorCode: number) {
    const message = getMultiDelegatorErrorMessage(
      errorCode as MultiDelegatorError,
    );
    super(message || `Program error: ${errorCode}`, 'PROGRAM_ERROR');
    this.name = 'ProgramError';
    this.errorCode = errorCode;
  }
}

/** Client-side validation failure (e.g. max destinations exceeded). */
export class ValidationError extends MultiDelegatorSDKError {
  constructor(message: string) {
    super(message, 'VALIDATION_ERROR');
    this.name = 'ValidationError';
  }
}
