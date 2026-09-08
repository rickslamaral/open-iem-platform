import type { Envelope, ClientMessage } from './types'
import { PROTOCOL_VERSION } from './types'

/** Generates a UUID v4 without external dependencies */
export function generateRequestId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  // Fallback: random hex segments in UUID v4 format
  const s4 = () =>
    Math.floor((1 + Math.random()) * 0x10000)
      .toString(16)
      .substring(1)
  return `${s4()}${s4()}-${s4()}-4${s4().slice(1)}-${((Math.random() * 4) | 8).toString(16)}${s4().slice(1)}-${s4()}${s4()}${s4()}`
}

/**
 * Wraps a ClientMessage in the wire Envelope and returns the envelope object.
 */
export function buildEnvelope(payload: ClientMessage): Envelope<ClientMessage> {
  return {
    version: PROTOCOL_VERSION,
    request_id: generateRequestId(),
    payload,
  }
}

/**
 * Wraps a ClientMessage in the wire Envelope and serialises to JSON string.
 */
export function encodeEnvelope(payload: ClientMessage): string {
  return JSON.stringify(buildEnvelope(payload))
}

/**
 * Parses a raw JSON string received from the server.
 * Returns null if the string is not valid JSON or doesn't look like an Envelope.
 */
export function decodeEnvelope<T>(raw: string): Envelope<T> | null {
  try {
    const obj = JSON.parse(raw) as unknown
    if (
      typeof obj === 'object' &&
      obj !== null &&
      'version' in obj &&
      'request_id' in obj &&
      'payload' in obj
    ) {
      return obj as Envelope<T>
    }
    return null
  } catch {
    return null
  }
}
