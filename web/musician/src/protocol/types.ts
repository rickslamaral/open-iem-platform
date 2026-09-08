// WebSocket envelope protocol types — MUST match server spec

export const PROTOCOL_VERSION = 1

export interface Envelope<T> {
  version: number // 1
  request_id: string // uuid v4
  payload: T
}

export type ClientMessage =
  | { type: 'GetState' }
  | { type: 'SetChannelGain'; data: { channel: number; gain_db: number } }
  | { type: 'SetChannelMute'; data: { channel: number; muted: boolean } }

export type ServerMessage =
  | { type: 'State'; data: { revision: number } }
  | { type: 'Error'; data: { code: string; message: string } }

// Auth API shapes
export interface LoginRequest {
  username: string
  password: string
}

export interface TokenResponse {
  access_token: string
  token_type: 'Bearer'
  expires_in: number
}

export type WsStatus = 'disconnected' | 'connecting' | 'connected' | 'error'
