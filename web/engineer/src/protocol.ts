// Engineer WebSocket protocol types — MUST match server control-protocol spec

export const PROTOCOL_VERSION = 1

export interface Envelope<T> {
  version: number
  request_id: string
  payload: T
}

export type ClientMessage =
  | { type: 'GetState' }
  | { type: 'SetMasterGain'; data: { mix_index: number; gain_db: number } }
  | { type: 'SetMasterMute'; data: { mix_index: number; muted: boolean } }

export type ServerMessage =
  | { type: 'State'; data: { revision: number } }
  | {
      type: 'MasterAck';
      data: {
        mix_index: number;
        master_gain_db: number;
        master_muted: boolean;
        revision: number;
      };
    }
  | { type: 'Error'; data: { code: string; message: string } }

export interface MixMasterState {
  master_gain_db: number
  master_muted: boolean
  revision: number
}

export type WsStatus = 'disconnected' | 'connecting' | 'connected' | 'error'
