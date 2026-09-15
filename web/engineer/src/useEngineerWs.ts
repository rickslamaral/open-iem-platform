import { useCallback, useEffect, useRef, useState } from 'react';
import {
  type ClientMessage,
  type Envelope,
  type EqBandState,
  type MixMasterState,
  type ServerMessage,
  type WsStatus,
  PROTOCOL_VERSION,
} from './protocol';

const WS_SUBPROTOCOL = 'openiem-v1';
const RECONNECT_DELAY_MS = 3000;
const MAX_MIX = 2;
const MAX_EQ_BANDS = 4;

function uuid(): string {
  return crypto.randomUUID();
}

const DEFAULT_EQ_BAND: EqBandState = {
  frequency_hz: 1000,
  gain_db: 0,
  q: 1.4,
  enabled: false,
};

function defaultEqBands(): EqBandState[][] {
  return Array.from({ length: MAX_MIX }, () =>
    Array.from({ length: MAX_EQ_BANDS }, () => ({ ...DEFAULT_EQ_BAND })),
  );
}

export interface UseEngineerWsResult {
  status: WsStatus;
  mixes: MixMasterState[];
  eqBands: EqBandState[][];
  revision: number | null;
  error: string | null;
  setMasterGain: (mixIndex: number, gainDb: number) => void;
  setMasterMute: (mixIndex: number, muted: boolean) => void;
  setEqBand: (mixIndex: number, bandIndex: number, params: Partial<EqBandState>) => void;
}

const DEFAULT_MIX: MixMasterState = { master_gain_db: 0, master_muted: false, revision: 0 };

/** Returns true when msg.data fields match a valid MasterAck shape. */
function isMasterAck(msg: ServerMessage): msg is ServerMessage & { type: 'MasterAck' } {
  return (
    msg.type === 'MasterAck' &&
    typeof (msg as { type: 'MasterAck'; data: Record<string, unknown> }).data === 'object' &&
    typeof (msg as { type: 'MasterAck'; data: { mix_index: unknown } }).data.mix_index === 'number'
  );
}

function isEqBandAck(msg: ServerMessage): msg is ServerMessage & { type: 'EqBandAck' } {
  return (
    msg.type === 'EqBandAck' &&
    typeof (msg as { type: 'EqBandAck'; data: Record<string, unknown> }).data === 'object' &&
    typeof (msg as { type: 'EqBandAck'; data: { mix_index: unknown } }).data.mix_index === 'number' &&
    typeof (msg as { type: 'EqBandAck'; data: { band_index: unknown } }).data.band_index === 'number'
  );
}

export function useEngineerWs(token: string | null): UseEngineerWsResult {
  const [status, setStatus] = useState<WsStatus>('disconnected');
  const [mixes, setMixes] = useState<MixMasterState[]>(
    Array.from({ length: MAX_MIX }, () => ({ ...DEFAULT_MIX })),
  );
  const [eqBands, setEqBands] = useState<EqBandState[][]>(defaultEqBands());
  // Ref mirrors eqBands state so setEqBand can read latest without stale closures
  const eqBandsRef = useRef<EqBandState[][]>(defaultEqBands());
  const [revision, setRevision] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const sendMsg = useCallback((msg: ClientMessage) => {
    const ws = wsRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    const envelope: Envelope<ClientMessage> = {
      version: PROTOCOL_VERSION,
      request_id: uuid(),
      payload: msg,
    };
    ws.send(JSON.stringify(envelope));
  }, []);

  const setMasterGain = useCallback(
    (mixIndex: number, gainDb: number) => {
      sendMsg({ type: 'SetMasterGain', data: { mix_index: mixIndex, gain_db: gainDb } });
    },
    [sendMsg],
  );

  const setMasterMute = useCallback(
    (mixIndex: number, muted: boolean) => {
      sendMsg({ type: 'SetMasterMute', data: { mix_index: mixIndex, muted } });
    },
    [sendMsg],
  );

  const setEqBand = useCallback(
    (mixIndex: number, bandIndex: number, params: Partial<EqBandState>) => {
      // Merge with current state from ref (avoids stale closure)
      const current = eqBandsRef.current[mixIndex]?.[bandIndex] ?? { ...DEFAULT_EQ_BAND };
      const merged: EqBandState = { ...current, ...params };
      // Update ref and state
      const next = eqBandsRef.current.map((mix) => [...mix]);
      next[mixIndex][bandIndex] = merged;
      eqBandsRef.current = next;
      setEqBands(next);
      // Send over WS
      sendMsg({
        type: 'SetEqBand',
        data: {
          mix_index: mixIndex,
          band_index: bandIndex,
          frequency_hz: merged.frequency_hz,
          gain_db: merged.gain_db,
          q: merged.q,
          enabled: merged.enabled,
        },
      });
    },
    [sendMsg],
  );

  useEffect(() => {
    if (!token) {
      wsRef.current?.close();
      wsRef.current = null;
      setStatus('disconnected');
      return;
    }

    let mounted = true;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

    function connect() {
      if (!mounted || !token) return;

      const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const url = `${proto}//${window.location.host}/ws/v1?access_token=${encodeURIComponent(token)}`;
      setStatus('connecting');
      setError(null);

      const ws = new WebSocket(url, WS_SUBPROTOCOL);
      wsRef.current = ws;

      ws.onopen = () => {
        if (!mounted) { ws.close(); return; }
        setStatus('connected');
        const getState: Envelope<ClientMessage> = {
          version: PROTOCOL_VERSION,
          request_id: uuid(),
          payload: { type: 'GetState' },
        };
        ws.send(JSON.stringify(getState));
      };

      ws.onmessage = (event: MessageEvent) => {
        if (!mounted) return;
        let envelope: Envelope<ServerMessage>;
        try {
          envelope = JSON.parse(String(event.data)) as Envelope<ServerMessage>;
        } catch {
          return;
        }
        const msg = envelope.payload;
        if (isMasterAck(msg)) {
          const idx = msg.data.mix_index;
          if (idx >= 0 && idx < MAX_MIX) {
            setMixes((prev) => {
              const next = [...prev];
              next[idx] = {
                master_gain_db: msg.data.master_gain_db,
                master_muted: msg.data.master_muted,
                revision: msg.data.revision,
              };
              return next;
            });
            setRevision(msg.data.revision);
          }
        } else if (isEqBandAck(msg)) {
          const { mix_index, band_index, frequency_hz, gain_db, q, enabled, revision: rev } = msg.data;
          if (mix_index >= 0 && mix_index < MAX_MIX && band_index >= 0 && band_index < MAX_EQ_BANDS) {
            const next = eqBandsRef.current.map((mix) => [...mix]);
            next[mix_index][band_index] = { frequency_hz, gain_db, q, enabled };
            eqBandsRef.current = next;
            setEqBands(next);
            setRevision(rev);
          }
        } else if (msg.type === 'State') {
          setRevision(msg.data.revision);
        } else if (msg.type === 'Error') {
          setError(msg.data.message);
        }
      };

      ws.onerror = () => {
        if (!mounted) return;
        setStatus('error');
        setError('Conexão WebSocket falhou.');
      };

      ws.onclose = () => {
        if (!mounted) return;
        setStatus('disconnected');
        reconnectTimer = setTimeout(connect, RECONNECT_DELAY_MS);
      };
    }

    connect();

    return () => {
      mounted = false;
      if (reconnectTimer !== null) clearTimeout(reconnectTimer);
      wsRef.current?.close();
      wsRef.current = null;
    };
  }, [token]);

  return { status, mixes, eqBands, revision, error, setMasterGain, setMasterMute, setEqBand };
}
