// WebSocket envelope protocol types — MUST match server spec

export const PROTOCOL_VERSION = 1

export interface Envelope<T> {
  version: number // 1
  request_id: string // uuid v4
  payload: T
}

export type ClientMessage =
  | { type: 'GetState' }
  | { type: 'SetSendGain'; data: { mix_index: number; channel_index: number; gain_db: number } }
  | { type: 'SetSendPan'; data: { mix_index: number; channel_index: number; pan: number } }
  | { type: 'SetSendMuted'; data: { mix_index: number; channel_index: number; muted: boolean } }
  | { type: 'SetChannelGain'; data: { channel: number; gain_db: number } }
  | { type: 'SetChannelMute'; data: { channel: number; muted: boolean } }

export interface StateSnapshot {
  schema_version: 1
  revision: number
  channels: Array<{
    index: number
    gain_db: number
    muted: boolean
  }>
  mixes: Array<{
    index: number
    master_gain_db: number
    master_muted: boolean
    sends: Array<{
      channel_index: number
      gain_db: number
      pan: number
      muted: boolean
    }>
  }>
}

export type ServerMessage =
  | { type: 'State'; data: { revision: number } }
  | {
      type: 'SendAck';
      data: {
        mix_index: number;
        channel_index: number;
        gain_db: number;
        pan: number;
        muted: boolean;
        revision: number;
      };
    }
  | {
      type: 'MasterAck';
      data: {
        mix_index: number;
        master_gain_db: number;
        master_muted: boolean;
        revision: number;
      };
    }
  | { type: 'Error'; data: { code: string; message: string } };

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
